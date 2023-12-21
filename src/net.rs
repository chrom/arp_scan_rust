use std::net::{IpAddr, Ipv4Addr};

use ipnetwork::Ipv4Network;

use pnet::datalink::{Channel, DataLinkReceiver, MacAddr, NetworkInterface};
use pnet::packet::arp::{ArpHardwareTypes, ArpOperations, ArpPacket, MutableArpPacket};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes, MutableEthernetPacket};
use pnet::packet::{MutablePacket, Packet};

pub fn get_available_interfaces<'a>(
    all_interfaces: &'a Vec<NetworkInterface>,
) -> Vec<&'a NetworkInterface> {
    all_interfaces
        .into_iter()
        .filter(|interface| interface.is_up() && !interface.is_loopback())
        .filter(|interface| interface.ips.iter().find(|ip| ip.is_ipv4()).is_some())
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

    let (mut sender, mut receiver) = match pnet::datalink::channel(&interface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unknown channel type"),
        Err(e) => panic!("Error happened {}", e),
    };

    let ethernet_packet = build_arp_packet(interface, source_ip);
    sender.send_to(ethernet_packet.packet(), None);
    receive_arp_responses(&mut receiver, &interface);
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



