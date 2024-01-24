use pnet::datalink::{MacAddr, NetworkInterface};
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// Displays a formatted list of available network interfaces.
///
/// This function takes a vector of references to `NetworkInterface` and prints a formatted
/// list of information for each interface, including the interface ID, name, MAC address,
/// IPv4 and IPv6 counts, and interface flags.
///
/// # Parameters
///
/// - `interfaces`: A vector of references to `NetworkInterface`.
///
/// # Examples
///
/// ```
/// use pnet::datalink::NetworkInterface;
/// use your_crate_name::show_list_interfaces;
///
/// fn main() {
///     let interfaces: Vec<NetworkInterface> = //...; // Obtain your network interfaces.
///
///     let interfaces_refs: Vec<&NetworkInterface> = interfaces.iter().collect();
///
///     show_list_interfaces(&interfaces_refs);
/// }
/// ```
pub fn show_list_interfaces(interfaces: &Vec<&NetworkInterface>) -> Result<(), std::io::Error> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout
        .set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
    writeln!(&mut stdout, "Available network interfaces:")?;

    for (id, interface) in interfaces.iter().enumerate() {
        // Id - first column
        colorize_and_write(&mut stdout, Color::Yellow, &format!("{}:", id));

        // Name - second column
        colorize_and_write(
            &mut stdout,
            Color::Cyan,
            &format!(
                " Name: {name:<max_name_length$}",
                name = interface.name,
                max_name_length = max_length(interfaces, |iface| iface.name.len())
            ),
        );

        colorize_and_write(
            &mut stdout,
            Color::White,
            &format!(
                " Mac: [{mac:<max_mac_length$}]",
                mac = interface.mac.unwrap_or(MacAddr::zero()).to_string(),
                max_mac_length = max_length(interfaces, |iface| {
                    iface.mac.map_or(0, |mac| mac.to_string().len())
                })
            ),
        );

        colorize_and_write(
            &mut stdout,
            Color::Magenta,
            &format!(
                " IPv4: [{ipv4:<max_ipv4_length$}]",
                ipv4 = interface
                    .ips
                    .iter()
                    .filter(|ip| ip.is_ipv4())
                    .map(|ip| ip.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                max_ipv4_length = max_length(&interfaces, get_max_ipv4_length)
            ),
        );

        colorize_and_write(
            &mut stdout,
            Color::Yellow,
            &format!(
                " Ipv6: [{ipv6:<max_ipv6_length$}]",
                ipv6 = interface
                    .ips
                    .iter()
                    .filter(|ip| ip.is_ipv6())
                    .map(|ip| ip.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                max_ipv6_length = max_length(interfaces, get_max_ipv6_length)
            ),
        );

        colorize_and_write(
            &mut stdout,
            Color::White,
            &format!(" Flags: [{flags}]", flags = get_flags(interface).unwrap()),
        );

        stdout.reset().unwrap();
        writeln!(&mut stdout).unwrap();
    }
    Ok(())
}

///
/// # Arguments
///
/// * `writer` - A mutable reference to a type implementing `WriteColor` where the content will be written.
/// * `color` - The color to be applied to the content.
/// * `content` - The text content to be colorized and written.
///
/// # Panics
///
/// This function will panic if setting the color or writing to the writer fails.
///
/// # Examples
///
/// ```rust
/// use termcolor::{StandardStream, ColorChoice, ColorSpec, WriteColor};
/// let mut stdout = StandardStream::stdout(ColorChoice::Always);
/// colorize_and_write(&mut stdout, Color::Green, "This is green text");
/// ```
fn colorize_and_write<W: WriteColor>(writer: &mut W, color: Color, content: &str) {
    writer
        .set_color(ColorSpec::new().set_fg(Some(color)))
        .unwrap();
    write!(writer, "{}", content).unwrap();
}

/// Finds the maximum length of a property among a collection of network interfaces.
///
/// # Arguments
///
/// * `interfaces` - A vector of references to `NetworkInterface` instances.
/// * `property` - A closure that takes a reference to a `NetworkInterface` and returns a property's length.
///
/// # Returns
///
/// The maximum length among the specified property for all network interfaces. Returns 0 if the vector is empty.
///
/// # Examples
///
/// ```rust
/// let interfaces = vec![/* ... */];
/// let max_name_length = max_length(&interfaces, |iface| iface.name.len());
/// ```
fn max_length<F: Fn(&NetworkInterface) -> usize>(
    interfaces: &Vec<&NetworkInterface>,
    property: F,
) -> usize {
    interfaces
        .iter()
        .map(|iface| property(iface))
        .max()
        .unwrap_or(0)
}

/// Gets a formatted string representing the flags of a network interface.
///
/// # Arguments
///
/// * `interface` - A reference to a `NetworkInterface` instance.
///
/// # Returns
///
/// A `Result` containing a `String` representing the flags of the network interface in hexadecimal format.
/// Returns an error string if there is an issue determining the flags.
///
/// # Examples
///
/// ```rust
/// let interface = /* ... */;
/// match get_flags(&interface) {
///     Ok(flags) => println!("Interface Flags: {}", flags),
///     Err(err) => eprintln!("Error getting interface flags: {}", err),
/// }
/// ```
fn get_flags(interface: &NetworkInterface) -> Result<String, String> {
    const FLAGS: [&str; 8] = [
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

fn get_max_ipv4_length(interface: &NetworkInterface) -> usize {
    interface
        .ips
        .iter()
        .filter(|ip| ip.is_ipv4())
        .map(|ip| ip.to_string().len())
        .max()
        .unwrap_or(0)
}

fn get_max_ipv6_length(interface: &NetworkInterface) -> usize {
    interface
        .ips
        .iter()
        .filter(|ip| ip.is_ipv6())
        .map(|ip| ip.to_string().len())
        .max()
        .unwrap_or(0)
}
