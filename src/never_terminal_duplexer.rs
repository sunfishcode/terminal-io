//! The `NeverTerminalDuplex` struct.

use crate::{DuplexTerminal, ReadTerminal, Terminal, TerminalColorSupport, WriteTerminal};
use duplex::{Duplex, HalfDuplex};
use std::{
    fmt,
    io::{self, IoSlice, IoSliceMut, Read, Write},
};
#[cfg(not(windows))]
use unsafe_io::os::posish::{AsRawReadWriteFd, RawFd};
#[cfg(windows)]
use unsafe_io::os::windows::{AsRawReadWriteHandleOrSocket, RawHandleOrSocket};
use unsafe_io::OwnsRaw;

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

// Safety: `NeverTerminalDuplexer` implements `OwnsRaw` if `Inner` does.
unsafe impl<Inner: Duplex + OwnsRaw> OwnsRaw for NeverTerminalDuplexer<Inner> {}

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
