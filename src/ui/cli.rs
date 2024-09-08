// External crate imports
use anstyle::{AnsiColor, Color::Ansi, Style};
use clap::builder::Styles as ClapStyles;

/// Defines custom styles for the Clap CLI interface.
///
/// This function configures and returns a `ClapStyles` object with custom
/// styling for various elements of the command-line interface. It uses ANSI
/// color codes to enhance the visual appearance and readability of the CLI.
///
/// # Styling Details
///
/// - Usage and Header: Bold, underlined, yellow text
/// - Literals: Green text
/// - Invalid input and Errors: Bold, red text
/// - Valid input: Bold, underlined, green text
/// - Placeholders: White text
///
/// # Returns
///
/// Returns a `ClapStyles` object with the custom styling applied.
///
pub fn style() -> ClapStyles {
    ClapStyles::styled()
        .usage(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Ansi(AnsiColor::Yellow))),
        )
        .header(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Ansi(AnsiColor::Yellow))),
        )
        .literal(Style::new().fg_color(Some(Ansi(AnsiColor::Green))))
        .invalid(Style::new().bold().fg_color(Some(Ansi(AnsiColor::Red))))
        .error(Style::new().bold().fg_color(Some(Ansi(AnsiColor::Red))))
        .valid(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Ansi(AnsiColor::Green))),
        )
        .placeholder(Style::new().fg_color(Some(Ansi(AnsiColor::White))))
}
