use crate::tools::{print_formatted_std_error, print_formatted_std_output};
use ipnetwork::Ipv4Network;
use pnet::datalink::NetworkInterface;
use std::io;
use termcolor::Color;

/// Retrieves the target IP address from the command-line arguments.
///
/// This function takes an iterator over command-line arguments and attempts to extract
/// the target IP address, assuming it is the second argument. It returns the parsed
/// `Ipv4Network` if successful.
///
/// # Parameters
///
/// - `args`: An iterator over command-line arguments (`String`).
///
/// # Returns
///
/// A `Result` containing the parsed `Ipv4Network` if successful, or a `String` error
/// message if there is an issue (e.g., missing target IP address or parsing error).
///
/// # Examples
///
/// ```
/// use std::env;
/// use std::net::Ipv4Network;
/// use your_crate_name::get_target_ip_from_args;
///
/// fn main() {
///     let args: Vec<String> = env::args().collect();
///
///     match get_target_ip_from_args(args.into_iter()) {
///         Ok(ip_target) => {
///             println!("Target IP: {}", ip_target);
///         }
///         Err(err) => {
///             eprintln!("Error: {}", err);
///         }
///     }
/// }
/// ```
pub fn get_target_ip_from_args(
    mut args: impl Iterator<Item = String>,
) -> Result<Ipv4Network, String> {
    let ip_target = args
        .nth(1)
        .ok_or_else(|| String::from("Missing target IP address"))?
        .parse::<Ipv4Network>()
        .map_err(|e| format!("Failed to parse IP address: {}", e))?;

    Ok(ip_target)
}

/// Prompts the user to select a network interface and returns the selected interface index.
///
/// This function takes a vector of references to `NetworkInterface` instances and prompts
/// the user to select an interface by entering the corresponding number. It returns the
/// index of the selected interface if the input is valid.
///
/// # Parameters
///
/// - `interfaces`: A reference to a vector of `NetworkInterface` instances.
///
/// # Returns
///
/// A `Result` containing the selected interface index if successful, or an `std::io::Error`
/// if there is an issue reading from the standard input.
///
/// # Examples
///
/// ```
/// use your_network_crate::NetworkInterface;
/// use your_crate_name::{print_formatted_std_output, print_formatted_std_error, prompt_for_interface};
/// use std::io;
///
/// // Assuming you have a vector of NetworkInterface instances named 'all_interfaces'
/// let available_interfaces = get_available_interfaces(&all_interfaces);
///
/// match prompt_for_interface(&available_interfaces) {
///     Ok(selected_index) => {
///         println!("Selected Interface: {}", available_interfaces[selected_index].name);
///     }
///     Err(err) => {
///         eprintln!("Error: {}", err);
///     }
/// }
/// ```
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
    use super::*;
    use std::net::Ipv4Addr;

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
        assert_eq!(
            result.err().unwrap(),
            "Failed to parse IP address: invalid address: invalid_ip"
        );
    }

    #[test]
    fn test_get_target_ip_from_args_invalid_subnet_mask() {
        let args = vec![
            "program_name".to_string(),
            "192.168.0.1/invalid_mask".to_string(),
        ];
        let result = get_target_ip_from_args(args.iter().cloned());

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            "Failed to parse IP address: invalid prefix"
        );
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
