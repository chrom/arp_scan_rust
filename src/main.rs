mod tools;

use std::env;
use std::io::{self, Write};
#[warn(unused_imports)]
use std::net::{AddrParseError, IpAddr, Ipv4Addr};
use std::process;

use nix::unistd::Uid;

use pnet::datalink;
use pnet::datalink::{Channel, DataLinkReceiver, MacAddr, NetworkInterface};

use pnet::packet::arp::{
    ArpHardwareTypes, ArpOperation, ArpOperations, ArpPacket, MutableArpPacket,
};
use pnet::packet::ethernet::MutableEthernetPacket;
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::{MutablePacket, Packet};

use ipnetwork::Ipv4Network;

use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use tools::*;

extern crate interfaces;

fn main() {
    if !Uid::effective().is_root() {
        tools::print_formatted_std_error(
            String::from("You must be root to run this program"),
            Some(Color::Red),
        );
        process::exit(1);
    }

    tools::check_supported_os().unwrap_or_else(|e| {
        tools::print_formatted_std_error(e, None);
        process::exit(1);
    });

    let target_ip = get_target_ip_from_args(env::args()).unwrap_or_else(|e| {
        tools::print_formatted_std_error(e, None);
        process::exit(1);
    });

    let binding = datalink::interfaces();

    // Get list of available network interfaces
    let interfaces = get_available_interfaces(&binding);
    show_list_interfaces(&interfaces);
    let index_interface = prompt_for_interface(&interfaces).unwrap_or_else(|e| {
        tools::print_formatted_std_error(e.to_string(), None);
        process::exit(1);
    });
    arp_scan(interfaces[index_interface], target_ip);
}

fn get_available_interfaces<'a>(
    all_interfaces: &'a Vec<NetworkInterface>,
) -> Vec<&'a NetworkInterface> {
    let interfaces = all_interfaces
        .iter()
        .filter(|e| e.is_up() && !e.is_loopback())
        .filter(|e| e.ips.iter().find(|ip| ip.is_ipv4()).is_some())
        .collect();
    interfaces
}

fn show_list_interfaces(interfaces: &Vec<&NetworkInterface>) {
    // calculate max length of interface name
    let max_name_length = interfaces
        .iter()
        .map(|iface| iface.name.len())
        .max()
        .unwrap_or(0);
    let max_mac_length = interfaces
        .iter()
        .map(|iface| iface.mac.map_or(0, |mac| mac.to_string().len()))
        .max()
        .unwrap_or(0);
    let max_ipv4_length = interfaces
        .iter()
        .flat_map(|iface| {
            iface
                .ips
                .iter()
                .filter(|ip| ip.is_ipv4())
                .map(|ip| ip.to_string().len())
        })
        .max()
        .unwrap_or(0);
    let max_ipv6_length = interfaces
        .iter()
        .flat_map(|iface| {
            iface
                .ips
                .iter()
                .filter(|ip| ip.is_ipv6())
                .map(|ip| ip.to_string().len())
        })
        .max()
        .unwrap_or(0);

    // Create an object StandardStream for standard output
    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    // Set colors for output
    stdout
        .set_color(ColorSpec::new().set_fg(Some(Color::Green)))
        .unwrap();
    writeln!(&mut stdout, "Available network interfaces:").unwrap();

    for (id, interface) in interfaces.iter().enumerate() {
        let all_v4_ips = interface
            .ips
            .iter()
            .filter(|ip| ip.is_ipv4())
            .map(|ip| ip.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let all_v6_ips = interface
            .ips
            .iter()
            .filter(|ip| ip.is_ipv6())
            .map(|ip| ip.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        // Set colors for different fields
        stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))
            .unwrap();
        write!(&mut stdout, "{}:", id).unwrap();

        stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))
            .unwrap();
        write!(
            &mut stdout,
            " Name: {name:<max_name_length$}",
            name = interface.name,
            max_name_length = max_name_length
        )
        .unwrap();

        stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::White)))
            .unwrap();
        write!(
            &mut stdout,
            " Mac: {mac:<max_mac_length$}",
            mac = interface.mac.map_or(MacAddr::zero(), |mac| mac.clone()),
            max_mac_length = max_mac_length
        )
        .unwrap();

        stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Magenta)))
            .unwrap();
        write!(
            &mut stdout,
            " IPv4: [{ipv4:<max_ipv4_length$}]",
            ipv4 = all_v4_ips,
            max_ipv4_length = max_ipv4_length
        )
        .unwrap();

        stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))
            .unwrap();
        write!(
            &mut stdout,
            " Ipv6: [{ipv6:<max_ipv6_length$}]",
            ipv6 = all_v6_ips,
            max_ipv6_length = max_ipv6_length
        )
        .unwrap();

        stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::White)))
            .unwrap();
        write!(
            &mut stdout,
            " Flags: [{flags}]",
            flags = get_flags(interface).unwrap()
        )
        .unwrap();

        // Reset colors
        stdout.reset().unwrap();

        // Move to the next line
        writeln!(&mut stdout).unwrap();
    }
}

fn prompt_for_interface(interfaces: &Vec<&NetworkInterface>) -> Result<usize, std::io::Error> {
    loop {
        tools::print_formatted_std_output(
            String::from("Please select the interface to use: "),
            Some(Color::Green),
        );

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if let Ok(interface_number) = input.trim().parse::<usize>() {
            if interface_number < interfaces.len() {
                return Result::Ok(interface_number);
            } else {
                tools::print_formatted_std_error(
                    String::from("Invalid interface number. Please enter a valid number: "),
                    Some(Color::Yellow),
                );
            }
        } else {
            tools::print_formatted_std_error(
                String::from("Invalid input. Please enter a valid number."),
                Some(Color::Yellow),
            );
        }
    }
}

fn arp_scan(interface: &NetworkInterface, _target_ip: Ipv4Network) {
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

fn get_target_ip_from_args(mut args: impl Iterator<Item = String>) -> Result<Ipv4Network, String> {
    let ip_target = args
        .nth(1)
        .ok_or_else(|| String::from("Missing target IP address"))?
        .parse::<Ipv4Network>()
        .map_err(|e| format!("Failed to parse IP address: {}", e))?;

    Ok(ip_target)
}

fn get_flags(interface: &NetworkInterface) -> Result<String, String> {
    const FLAGS: [&'static str; 8] = [
        "UP",
        "BROADCAST",
        "LOOPBACK",
        "POINTOPOINT",
        "MULTICAST",
        "RUNNING",
        "DORMANT",
        "LOWERUP",
    ];
    let flags = if interface.flags > 0 {
        #[cfg(any(target_os = "linux", target_os = "android"))]
        let rets = [
            interface.is_up(),
            interface.is_broadcast(),
            interface.is_loopback(),
            interface.is_point_to_point(),
            interface.is_multicast(),
            interface.is_running(),
            interface.is_dormant(),
            interface.is_lower_up(),
        ];
        #[cfg(all(unix, not(any(target_os = "linux", target_os = "android"))))]
        let rets = [
            interface.is_up(),
            interface.is_broadcast(),
            interface.is_loopback(),
            interface.is_point_to_point(),
            interface.is_multicast(),
            interface.is_running(),
            false,
            false,
        ];
        #[cfg(not(unix))]
        let rets = [
            interface.is_up(),
            interface.is_broadcast(),
            interface.is_loopback(),
            interface.is_point_to_point(),
            interface.is_multicast(),
            false,
            false,
            false,
        ];

        format!(
            "{:X}<{}>",
            interface.flags,
            rets.iter()
                .zip(FLAGS.iter())
                .filter(|&(ret, _)| ret == &true)
                .map(|(_, name)| name.to_string())
                .collect::<Vec<String>>()
                .join(",")
        )
    } else {
        format!("{:X}", interface.flags)
    };

    Ok(flags)
}
