use std::io;
use std::io::Write;

use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn check_supported_os() -> Result<(), String> {
    return match os_version::detect() {
        Ok(os) => match os {
            os_version::OsVersion::Linux(_) => Ok(()),
            _ => Err(String::from("Only OS Linux is supported")),
        },
        Err(e) => Err(format!("Failed to detect OS version: {}", e)),
    };
}

pub fn print_formatted_std_error(error: String, color: Option<Color>) {
    let mut stderr = StandardStream::stderr(ColorChoice::Always);
    stderr
        .set_color(ColorSpec::new().set_fg(Some(color.unwrap_or(Color::Red))))
        .unwrap();
    writeln!(&mut stderr, "{}", error).unwrap();
    stderr.reset().unwrap();
}

/// Prints an error message to stderr
///
/// # Arguments
///
/// * `error`:
/// * `color`:
///
/// returns: ()
///
/// # Examples
///
/// ```
/// print_formatted_error("Invalid input. Please enter a valid number.".to_string(), Some(Color::Yellow));
/// print_formatted_error("Invalid input. Please enter a valid number.".to_string(), None);
/// ```

fn print_formatted_std_output(output: String, color: Option<Color>) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout
        .set_color(ColorSpec::new().set_fg(Some(color.unwrap_or(Color::Green))))
        .unwrap();
    writeln!(&mut stdout, "{}", output).unwrap();
    stdout.reset().unwrap();
    let _ = io::stdout().flush();
}
