//! The `NeverTerminalDuplex` struct.

use crate::{DuplexTerminal, ReadTerminal, Terminal, TerminalColorSupport, WriteTerminal};
use duplex::{Duplex, HalfDuplex};
#[cfg(windows)]
use io_extras::os::windows::{
    AsRawReadWriteHandleOrSocket, AsReadWriteHandleOrSocket, BorrowedHandleOrSocket,
    RawHandleOrSocket,
};
use std::fmt;
use std::io::{self, IoSlice, IoSliceMut, Read, Write};
#[cfg(not(windows))]
use {
    io_extras::os::rustix::{AsRawReadWriteFd, AsReadWriteFd, RawFd},
    io_lifetimes::BorrowedFd,
};

/// A wrapper around a `Read` + `Write` which implements `DuplexTerminal`
/// but isn't ever a terminal.
#[derive(Debug)]
pub struct NeverTerminalDuplexer<Inner: Duplex> {
    inner: Inner,
}

impl<Inner: Duplex> NeverTerminalDuplexer<Inner> {
    /// Wrap a `TerminalReader` around the given stream.
    pub fn new(inner: Inner) -> Self {
        Self { inner }
    }

    /// Consume `self` and return the inner stream.
    #[inline]
    pub fn into_inner(self) -> Inner {
        self.inner
    }
}

#[cfg(not(windows))]
impl<Inner: Duplex + AsRawReadWriteFd> AsRawReadWriteFd for NeverTerminalDuplexer<Inner> {
    #[inline]
    fn as_raw_read_fd(&self) -> RawFd {
        self.inner.as_raw_read_fd()
    }

    #[inline]
    fn as_raw_write_fd(&self) -> RawFd {
        self.inner.as_raw_write_fd()
    }
}

#[cfg(not(windows))]
impl<Inner: Duplex + AsReadWriteFd> AsReadWriteFd for NeverTerminalDuplexer<Inner> {
    #[inline]
    fn as_read_fd(&self) -> BorrowedFd<'_> {
        self.inner.as_read_fd()
    }

    #[inline]
    fn as_write_fd(&self) -> BorrowedFd<'_> {
        self.inner.as_write_fd()
    }
}

#[cfg(windows)]
impl<Inner: Duplex + AsRawReadWriteHandleOrSocket> AsRawReadWriteHandleOrSocket
    for NeverTerminalDuplexer<Inner>
{
    #[inline]
    fn as_raw_read_handle_or_socket(&self) -> RawHandleOrSocket {
        self.inner.as_raw_read_handle_or_socket()
    }

    #[inline]
    fn as_raw_write_handle_or_socket(&self) -> RawHandleOrSocket {
        self.inner.as_raw_write_handle_or_socket()
    }
}

#[cfg(windows)]
impl<Inner: Duplex + AsReadWriteHandleOrSocket> AsReadWriteHandleOrSocket
    for NeverTerminalDuplexer<Inner>
{
    #[inline]
    fn as_read_handle_or_socket(&self) -> BorrowedHandleOrSocket<'_> {
        self.inner.as_read_handle_or_socket()
    }

    #[inline]
    fn as_write_handle_or_socket(&self) -> BorrowedHandleOrSocket<'_> {
        self.inner.as_write_handle_or_socket()
    }
}

impl<Inner: Duplex> Terminal for NeverTerminalDuplexer<Inner> {}

impl<Inner: Duplex + Read> ReadTerminal for NeverTerminalDuplexer<Inner> {
    fn is_line_by_line(&self) -> bool {
        false
    }

    fn is_input_terminal(&self) -> bool {
        false
    }
}

impl<Inner: Duplex + Write> WriteTerminal for NeverTerminalDuplexer<Inner> {
    fn color_support(&self) -> TerminalColorSupport {
        TerminalColorSupport::default()
    }

    fn color_preference(&self) -> bool {
        false
    }

    fn is_output_terminal(&self) -> bool {
        false
    }
}

impl<Inner: Duplex + HalfDuplex> DuplexTerminal for NeverTerminalDuplexer<Inner> {}

impl<Inner: Duplex + Read> Read for NeverTerminalDuplexer<Inner> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut]) -> io::Result<usize> {
        self.inner.read_vectored(bufs)
    }

    #[cfg(can_vector)]
    #[inline]
    fn is_read_vectored(&self) -> bool {
        self.inner.is_read_vectored()
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.inner.read_to_end(buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        self.inner.read_to_string(buf)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.inner.read_exact(buf)
    }
}

impl<Inner: Duplex + Write> Write for NeverTerminalDuplexer<Inner> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice]) -> io::Result<usize> {
        self.inner.write_vectored(bufs)
    }

    #[cfg(can_vector)]
    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.inner.is_write_vectored()
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.inner.write_all(buf)
    }

    #[cfg(write_all_vectored)]
    #[inline]
    fn write_all_vectored(&mut self, bufs: &mut [IoSlice]) -> io::Result<()> {
        self.inner.write_all_vectored(bufs)
    }

    #[inline]
    fn write_fmt(&mut self, fmt: fmt::Arguments) -> io::Result<()> {
        self.inner.write_fmt(fmt)
    }
}

impl<Inner: Duplex> Duplex for NeverTerminalDuplexer<Inner> {}
