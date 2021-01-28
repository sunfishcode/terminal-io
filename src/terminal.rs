use duplex::Duplex;
use std::io::{Read, Write};

/// A trait for devices which may be connected to terminals.
pub trait Terminal {}

/// An extension trait for input streams connected to terminals.
pub trait ReadTerminal: Read + Terminal {
    /// Test whether the input is being sent a line at a time.
    fn is_line_by_line(&self) -> bool;

    /// Test whether the input is connected to a terminal.
    ///
    /// Also known as `isatty`.
    fn is_input_terminal(&self) -> bool;
}

/// An extension trait for output streams connected to terminals.
pub trait WriteTerminal: Write + Terminal {
    /// Test whether color should be used on this terminal by default. This
    /// includes both whether color is supported and whether the user has
    /// not indicated a preference otherwise.
    fn color_default(&self) -> bool {
        self.color_support() != TerminalColorSupport::Monochrome && self.color_preference()
    }

    /// Test whether this output stream supports color control codes.
    fn color_support(&self) -> TerminalColorSupport;

    /// Test whether the user has indicated a preference for color output by
    /// default. Respects the `NO_COLOR` environment variable where applicable.
    fn color_preference(&self) -> bool;

    /// Test whether the output is connected to a terminal.
    ///
    /// Also known as `isatty`.
    fn is_output_terminal(&self) -> bool;
}

/// An extension trait for input/output streams connected to terminals.
pub trait DuplexTerminal: ReadTerminal + WriteTerminal + Duplex {
    /// Test whether both the input stream and output streams are connected to
    /// terminals.
    ///
    /// Also known as `isatty`.
    fn is_terminal(&self) -> bool {
        self.is_input_terminal() && self.is_output_terminal()
    }
}

/// Color support level, ranging from monochrome (color not supported) to
/// 24-bit true color.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TerminalColorSupport {
    /// Color is not supported.
    Monochrome,

    /// Classic ANSI 8 colors. Sometimes extendable to 16 by using bold.
    Classic8,

    /// 256 colors with a "color cube". See [Wikipedia] for details.
    ///
    /// [Wikipedia]: https://en.wikipedia.org/wiki/ANSI_escape_code#8-bit
    ColorCube256,

    /// 24-bit "true color" support. See [Wikipedia] for details.
    ///
    /// [Wikipedia]: https://en.wikipedia.org/wiki/ANSI_escape_code#24-bit
    TrueColor,
}

impl Default for TerminalColorSupport {
    #[inline]
    fn default() -> Self {
        Self::Monochrome
    }
}
