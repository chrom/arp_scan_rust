mod tools;
mod net;
mod cli;

mod view {
    pub mod row;
}

use std::env;
use std::process;

use nix::unistd::Uid;

use pnet::datalink;

use termcolor::{Color};

use tools::{check_supported_os, print_formatted_std_error};

extern crate interfaces;

fn main() {
    if !Uid::effective().is_root() {
        print_formatted_std_error(
            String::from("You must be root privilege to run this program"),
            Some(Color::Red),
        );
        process::exit(1)
    }

    check_supported_os().unwrap_or_else(|e| {
        print_formatted_std_error(e, None);
        process::exit(1)
    });

    let target_ip = cli::get_target_ip_from_args(env::args()).unwrap_or_else(|e| {
        print_formatted_std_error(e, None);
        process::exit(1)
    });

    let binding = datalink::interfaces();

    // Get list of available network interfaces
    let interfaces = net::get_available_interfaces(&binding);

    view::row::show_list_interfaces(&interfaces);

    let index_interface = cli::prompt_for_interface(&interfaces).unwrap_or_else(|e| {
        print_formatted_std_error(e.to_string(), None);
        process::exit(1)
    });
    net::arp_scan(interfaces[index_interface], target_ip);
}

