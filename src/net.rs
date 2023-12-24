use std::net::{IpAddr, Ipv4Addr};

use ipnetwork::Ipv4Network;

use pnet::datalink::{Channel, DataLinkReceiver, MacAddr, NetworkInterface};
use pnet::packet::arp::{ArpHardwareTypes, ArpOperations, ArpPacket, MutableArpPacket};
use pnet::packet::ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket};
use pnet::packet::{MutablePacket, Packet};

/// Returns a vector of references to available network interfaces.
///
/// This function takes a reference to a vector of `NetworkInterface` instances and
/// filters out interfaces that are not up, are loopback, or do not have any IPv4 addresses.
/// The remaining interfaces are collected into a new vector and returned.
///
/// # Parameters
///
/// - `all_interfaces`: A reference to a vector of `NetworkInterface` instances.
///
/// # Returns
///
/// A vector of references to available network interfaces.
///
/// # Examples
///
/// ```
/// use your_network_crate::{NetworkInterface, IpAddress};
/// use your_crate_name::get_available_interfaces;
///
/// // Assuming you have a vector of NetworkInterface instances named 'all_interfaces'
/// let available_interfaces = get_available_interfaces(&all_interfaces);
/// for interface in available_interfaces {
///     println!("Available Interface: {}", interface.name);
/// }
/// ```
pub fn get_available_interfaces(all_interfaces: &Vec<NetworkInterface>) -> Vec<&NetworkInterface> {
    all_interfaces
        .iter()
        .filter(|interface| interface.is_up() && !interface.is_loopback())
        .filter(|interface| interface.ips.iter().any(|ip| ip.is_ipv4()))
        .collect()
}

pub fn arp_scan(interface: &NetworkInterface, _target_ip: Ipv4Network) {
    let source_ip = interface
        .ips
        .iter()
        .find(|ip| ip.is_ipv4())
        .map(|ip| match ip.ip() {
            IpAddr::V4(ip) => ip,
            _ => unreachable!(),
        })
        .unwrap();

    let (mut sender, mut receiver) = match pnet::datalink::channel(interface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unknown channel type"),
        Err(e) => panic!("Error happened {}", e),
    };

    let ethernet_packet = build_arp_packet(interface, source_ip);
    sender.send_to(ethernet_packet.packet(), None);
    receive_arp_responses(&mut receiver, interface);
}

fn receive_arp_responses(receiver: &mut Box<dyn DataLinkReceiver>, interface: &NetworkInterface) {
    loop {
        match receiver.next() {
            Ok(packet) => {
                if let Some(ethernet) = EthernetPacket::new(packet) {
                    if ethernet.get_ethertype() == pnet::packet::ethernet::EtherTypes::Arp {
                        if let Some(arp) = ArpPacket::new(ethernet.payload()) {
                            if arp.get_operation() == ArpOperations::Reply
                                && arp.get_sender_proto_addr() == interface.ips[0].ip()
                            {
                                println!("{:?}", arp);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {:?}", e);
            }
        }
    }
}

fn build_arp_packet(interface: &NetworkInterface, source_ip: Ipv4Addr) -> MutableEthernetPacket {
    let mut ethernet_packet = MutableEthernetPacket::owned(vec![0u8; 42]).unwrap();

    ethernet_packet.set_destination(MacAddr::broadcast());
    ethernet_packet.set_source(interface.mac.unwrap());
    ethernet_packet.set_ethertype(EtherTypes::Arp);

    let mut arp_buffer = [0u8; 28];
    let mut arp_packet = MutableArpPacket::new(&mut arp_buffer).unwrap();

    arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
    arp_packet.set_protocol_type(EtherTypes::Ipv4);
    arp_packet.set_hw_addr_len(6);
    arp_packet.set_proto_addr_len(4);
    arp_packet.set_operation(ArpOperations::Request);
    arp_packet.set_sender_hw_addr(interface.mac.unwrap());
    arp_packet.set_sender_proto_addr(source_ip);
    arp_packet.set_target_hw_addr(MacAddr::zero());
    arp_packet.set_target_proto_addr(Ipv4Addr::new(0, 0, 0, 0));

    ethernet_packet.set_payload(arp_packet.packet_mut());
    ethernet_packet
}
