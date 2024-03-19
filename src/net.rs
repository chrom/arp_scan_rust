use std::io::Error;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

use pnet::datalink::{Channel, Config, DataLinkReceiver, MacAddr, NetworkInterface};
use pnet::packet::{MutablePacket, Packet};
use pnet::packet::arp::{ArpHardwareTypes, ArpOperations, ArpPacket, MutableArpPacket};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes, MutableEthernetPacket};

use crate::options::CliOptions;

pub const DATALINK_RCV_TIMEOUT: u64 = 500;


/// Returns a vector of references to available network interfaces.
///
/// This function takes a reference to a vector of `NetworkInterface` instances and
/// filters out interfaces that are not up, are loopback, or do not have any IPv4 addresses.
/// ARP (Address Resolution Protocol) is used to map a known IP address to a MAC (Media Access Control) address in IPv4 networks.
/// For IPv6, the equivalent protocol is NDP (Neighbor Discovery Protocol).
/// NDP serves the same purpose as ARP but is designed specifically for IPv6.
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


fn get_source_ip_from_interface(interface: &NetworkInterface) -> Result<Ipv4Addr, Error> {
    let source_ip = interface
        .ips
        .iter()
        .find(|ip| ip.is_ipv4())
        .map(|ip| match ip.ip() {
            IpAddr::V4(ip) => ip,
            _ => unreachable!(),
        }).ok_or(std::io::Error::new(std::io::ErrorKind::Other, format!("No IPv4 address found in interface: {}", interface.name)))?;
    Ok(source_ip)
}

pub fn arp_scan(interface: &NetworkInterface, _options: &CliOptions) -> std::result::Result<(), std::io::Error> {
    let source_ip = get_source_ip_from_interface(interface)?;
    // dbg!(source_ip);
    // process::exit(0x0100);


    let (mut sender, mut receiver) = match pnet::datalink::channel(interface, get_channel_config()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unknown channel type"),
        Err(e) => Err(e)?,
    };

    sender.send_to(build_arp_packet(interface, source_ip).packet(), None);

    receive_arp_responses(&mut receiver, interface);

    Ok(())
}


/// Returns the maximum value of a property of a vector of `NetworkInterface` instances.
/// This function takes a reference to a vector of `NetworkInterface` instances and a closure
/// that returns a value of type `T` for each `NetworkInterface` instance. It returns the
/// maximum value of type `T` from all the values returned by the closure.
fn get_channel_config() -> Config {
    Config {
        read_timeout: Some(Duration::from_millis(DATALINK_RCV_TIMEOUT)),
        ..Config::default()
    }
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
