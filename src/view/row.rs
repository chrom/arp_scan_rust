    use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
    use pnet::datalink::{MacAddr, NetworkInterface};
    use std::io::Write;

    pub fn show_list_interfaces(interfaces: &Vec<&NetworkInterface>) {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Green)))
            .unwrap();
        writeln!(&mut stdout, "Available network interfaces:").unwrap();

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
                    " Mac: {mac:<max_mac_length$}",
                    mac = interface.mac.unwrap_or(MacAddr::zero()).to_string(),
                    max_mac_length = max_length(interfaces, |iface| {
                        iface.mac.map_or(0, |mac| mac.to_string().len())
                    })
                ),
            );

            let all_v4_ips = interface
                .ips
                .iter()
                .filter(|ip| ip.is_ipv4())
                .count();
            colorize_and_write(
                &mut stdout,
                Color::Magenta,
                &format!(
                    " IPv4: [{ipv4:<max_ipv4_length$}]",
                    ipv4 = all_v4_ips,
                    max_ipv4_length = max_length(interfaces, |iface| {
                        iface.ips.iter().filter(|ip| ip.is_ipv4()).count()
                    })
                ),
            );

            let all_v6_ips = interface
                .ips
                .iter()
                .filter(|ip| ip.is_ipv6())
                .count();
            colorize_and_write(
                &mut stdout,
                Color::Yellow,
                &format!(
                    " Ipv6: [{ipv6:<max_ipv6_length$}]",
                    ipv6 = all_v6_ips,
                    max_ipv6_length = max_length(interfaces, |iface| {
                        iface.ips.iter().filter(|ip| ip.is_ipv6()).count()
                    })
                ),
            );

            colorize_and_write(
                &mut stdout,
                Color::White,
                &format!(
                    " Flags: [{flags}]",
                    flags = get_flags(interface).unwrap()
                ),
            );

            stdout.reset().unwrap();
            writeln!(&mut stdout).unwrap();
        }
    }

    fn colorize_and_write<W: WriteColor>(
        writer: &mut W,
        color: Color,
        content: &str,
    ) {
        writer.set_color(ColorSpec::new().set_fg(Some(color))).unwrap();
        write!(writer, "{}", content).unwrap();
    }

    fn max_length<F: Fn(&NetworkInterface) -> usize>(
        interfaces: &Vec<&NetworkInterface>,
        property: F,
    ) -> usize {
        interfaces.iter().map(|iface| property(*iface)).max().unwrap_or(0)
    }

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
