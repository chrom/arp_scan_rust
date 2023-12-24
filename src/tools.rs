use std::io;
use std::io::Write;

use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// Checks if the current operating system is supported.
///
/// This function uses the `os_version` crate to detect the operating system.
/// If the operating system is Linux, it returns `Ok(())`, indicating that the
/// OS is supported. Otherwise, it returns an `Err` with a descriptive error message.
///
/// # Errors
///
/// Returns an `Err` variant with a string describing the error in the following cases:
/// - If the OS is not Linux.
/// - If there is an error while detecting the OS version.
///
/// # Examples
///
/// ```
/// use your_crate_name::check_supported_os;
///
/// match check_supported_os() {
///     Ok(()) => println!("The operating system is supported."),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn check_supported_os() -> Result<(), String> {
    match os_version::detect() {
        Ok(os) => match os {
            os_version::OsVersion::Linux(_) => Ok(()),
            _ => Err(String::from("Only OS Linux is supported")),
        },
        Err(e) => Err(format!("Failed to detect OS version: {}", e)),
    }
}


/// Prints a formatted error message to the standard error stream.
///
/// This function takes an error message as a `String` and an optional `Color`
/// to specify the text color. If no color is provided, it defaults to `Color::Red`.
///
/// # Parameters
///
/// - `error`: A string containing the error message to be printed.
/// - `color`: An optional `Color` enum value specifying the text color. Defaults to `Color::Red` if not provided.
///
/// # Examples
///
/// ```
/// use termcolor::Color;
/// use your_crate_name::print_formatted_std_error;
///
/// // Print a red error message
/// print_formatted_std_error(String::from("An error occurred"), Some(Color::Red));
///
/// // Print a default red error message
/// print_formatted_std_error(String::from("Another error"));
/// ```
pub fn print_formatted_std_error(error: String, color: Option<Color>) {
    let mut stderr = StandardStream::stderr(ColorChoice::Always);
    stderr
        .set_color(ColorSpec::new().set_fg(Some(color.unwrap_or(Color::Red))))
        .unwrap();
    stderr.write_fmt(format_args!("{0}\n", error)).unwrap();
    stderr.reset().unwrap();
}

/// Prints a formatted message to the standard output stream.
///
/// This function takes a message as a `String` and an optional `Color`
/// to specify the text color. If no color is provided, it defaults to `Color::Green`.
///
/// # Parameters
///
/// - `msg`: A string containing the message to be printed.
/// - `color`: An optional `Color` enum value specifying the text color. Defaults to `Color::Green` if not provided.
///
/// # Examples
///
/// ```
/// use termcolor::Color;
/// use your_crate_name::print_formatted_std_output;
///
/// // Print a green message
/// print_formatted_std_output(String::from("Operation successful"), Some(Color::Green));
///
/// // Print a default green message
/// print_formatted_std_output(String::from("Another message"));
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
