use std::io;
use std::io::Write;

use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn check_supported_os() -> Result<(), String> {
    match os_version::detect() {
        Ok(os) => {
            match os {
                os_version::OsVersion::Linux(_) => Ok(()),
                _ => Err(String::from("Only OS Linux is supported")),
            }
        },
        Err(e) => Err(format!("Failed to detect OS version: {}", e)),
    }
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
///
/// ```
pub fn print_formatted_std_error(error: String, color: Option<Color>) {
    let mut stderr = StandardStream::stderr(ColorChoice::Always);
    stderr
        .set_color(ColorSpec::new().set_fg(Some(color.unwrap_or(Color::Red))))
        .unwrap();
    stderr.write_fmt(format_args!("{0}\n", error)).unwrap();
    stderr.reset().unwrap();
}

/// Prints an error message to output
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
/// print_formatted_std_output("Scanning started".to_string(), Some(Color::Yellow));
/// print_formatted_std_output("Scanning finished.".to_string(), None);
/// ```

pub fn print_formatted_std_output(msg: String, color: Option<Color>) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout
        .set_color(ColorSpec::new().set_fg(Some(color.unwrap_or(Color::Green))))
        .unwrap();
    stdout.write_fmt(format_args!("{0}\n", msg)).unwrap();
    stdout.reset().unwrap();
    let _ = io::stdout().flush();
}
