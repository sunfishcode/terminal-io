//! Utilities for reading and writing on terminals.

#![deny(missing_docs)]
#![cfg_attr(can_vector, feature(can_vector))]
#![cfg_attr(write_all_vectored, feature(write_all_vectored))]

mod config;
mod never_terminal_duplexer;
mod never_terminal_reader;
mod never_terminal_writer;
mod terminal;
mod terminal_duplexer;
mod terminal_reader;
mod terminal_writer;

pub use never_terminal_duplexer::NeverTerminalDuplexer;
pub use never_terminal_reader::NeverTerminalReader;
pub use never_terminal_writer::NeverTerminalWriter;
pub use terminal::{DuplexTerminal, ReadTerminal, Terminal, TerminalColorSupport, WriteTerminal};
pub use terminal_duplexer::TerminalDuplexer;
pub use terminal_reader::TerminalReader;
pub use terminal_writer::TerminalWriter;
