extern crate interfaces;

use std::process;

use nix::unistd::Uid;
use pnet::datalink;
use termcolor::Color;

use tools::{check_supported_os, print_formatted_std_error};

mod cli;
mod net;
mod tools;
mod options;

mod view {
    pub mod plain;
}

fn main() {
    if !Uid::effective().is_root() {
        print_formatted_std_error(
            String::from("You must be root privilege to run this program"),
            Some(Color::Red),
        );
        process::exit(exitcode::NOPERM);
    }

    check_supported_os().unwrap_or_else(|e| {
        print_formatted_std_error(e, None);
        process::exit(exitcode::OSERR);
    });

    let command = cli::build_command().get_matches();
    let scan_options = options::CliOptions::new(&command).unwrap_or_else(|e| {
        print_formatted_std_error(e.to_string(), None);
        process::exit(exitcode::USAGE);
    });

    let binding = datalink::interfaces();

    // Get list of available network interfaces
    let interfaces = net::get_available_interfaces(&binding);

    view::plain::show_list_interfaces(&interfaces).unwrap_or_else(|e| {
        print_formatted_std_error(e.to_string(), None);
        process::exit(exitcode::UNAVAILABLE);
    });

    let selected_interface = cli::prompt_for_interface(&interfaces).unwrap_or_else(|e| {
        print_formatted_std_error(e.to_string(), None);
        process::exit(exitcode::USAGE);
    });

    net::arp_scan(interfaces[selected_interface], &scan_options).unwrap_or_else(|e| {
        print_formatted_std_error(e.to_string(), None);
        process::exit(exitcode::UNAVAILABLE);
    });

    process::exit(exitcode::OK);
}
