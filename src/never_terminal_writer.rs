//! The `NeverTerminalWriter` struct.

use crate::{Terminal, TerminalColorSupport, WriteTerminal};
use std::{
    fmt,
    io::{self, IoSlice, Write},
};
#[cfg(not(windows))]
use unsafe_io::os::posish::{AsRawFd, RawFd};
#[cfg(windows)]
use unsafe_io::os::windows::{AsRawHandleOrSocket, RawHandleOrSocket};

/// A wrapper around a `Write` which implements `WriteTerminal` but isn't ever
/// a terminal.
#[derive(Debug)]
pub struct NeverTerminalWriter<Inner: Write> {
    inner: Inner,
}

impl<Inner: Write> NeverTerminalWriter<Inner> {
    /// Wrap a `NeverTerminalWriter` around the given stream.
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
impl<Inner: Write + AsRawFd> AsRawFd for NeverTerminalWriter<Inner> {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

#[cfg(windows)]
impl<Inner: Write + AsRawHandleOrSocket> AsRawHandleOrSocket for NeverTerminalWriter<Inner> {
    #[inline]
    fn as_raw_handle_or_socket(&self) -> RawHandleOrSocket {
        self.inner.as_raw_handle_or_socket()
    }
}

impl<Inner: Write> Terminal for NeverTerminalWriter<Inner> {}

impl<Inner: Write> WriteTerminal for NeverTerminalWriter<Inner> {
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

impl<Inner: Write> Write for NeverTerminalWriter<Inner> {
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
