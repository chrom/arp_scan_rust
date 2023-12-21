use std::io;
use ipnetwork::Ipv4Network;
use pnet::datalink::NetworkInterface;
use termcolor::Color;
use crate::tools::{print_formatted_std_error, print_formatted_std_output};


///
///
/// # Arguments
///
/// * `args`:
///
/// returns: Result<Ipv4Network, String>
///
/// # Examples
///
/// ```
///
/// ```
pub fn get_target_ip_from_args(mut args: impl Iterator<Item = String>) -> Result<Ipv4Network, String> {
    let ip_target = args
        .nth(1)
        .ok_or_else(|| String::from("Missing target IP address"))?
        .parse::<Ipv4Network>()
        .map_err(|e| format!("Failed to parse IP address: {}", e))?;

    Ok(ip_target)
}

pub fn prompt_for_interface(interfaces: &Vec<&NetworkInterface>) -> Result<usize, std::io::Error> {
    loop {
        print_formatted_std_output(
            String::from("Please select the interface to use: "),
            Some(Color::Green),
        );

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if let Ok(interface_number) = input.trim().parse::<usize>() {
            if interface_number < interfaces.len() {
                return Result::Ok(interface_number);
            } else {
                print_formatted_std_error(
                    String::from("Invalid interface number. Please enter a valid number: "),
                    Some(Color::Yellow),
                );
            }
        } else {
            print_formatted_std_error(
                String::from("Invalid input. Please enter a valid number."),
                Some(Color::Yellow),
            );
        }
    }
}


#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;
    use super::*;

    #[test]
    fn test_get_target_ip_from_args_no_args() {
        let args = Vec::<String>::new();
        let result = get_target_ip_from_args(args.iter().cloned());

        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "Missing target IP address");
    }

    #[test]
    fn test_get_target_ip_from_args_insufficient_args() {
        let args = vec!["program_name".to_string()];
        let result = get_target_ip_from_args(args.iter().cloned());

        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "Missing target IP address");
    }

    #[test]
    fn test_get_target_ip_from_args_invalid_ip_format() {
        let args = vec!["program_name".to_string(), "invalid_ip".to_string()];
        let result = get_target_ip_from_args(args.iter().cloned());

        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "Failed to parse IP address: invalid address: invalid_ip");
    }

    #[test]
    fn test_get_target_ip_from_args_invalid_subnet_mask() {
        let args = vec!["program_name".to_string(), "192.168.0.1/invalid_mask".to_string()];
        let result = get_target_ip_from_args(args.iter().cloned());

        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "Failed to parse IP address: invalid prefix");
    }

    #[test]
    fn test_get_target_ip_from_args_valid_args() {
        let args = vec!["program_name".to_string(), "192.168.0.1/24".to_string()];
        let result = get_target_ip_from_args(args.iter().cloned());

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Ipv4Network::new(Ipv4Addr::new(192, 168, 0, 1), 24).unwrap()
        );
    }
}
