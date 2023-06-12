//! The `NeverTerminalReader` struct.

use crate::{ReadTerminal, Terminal};
#[cfg(windows)]
use io_extras::os::windows::{
    AsHandleOrSocket, AsRawHandleOrSocket, BorrowedHandleOrSocket, RawHandleOrSocket,
};
use std::io::{self, IoSliceMut, Read};
#[cfg(not(windows))]
use {
    io_extras::os::rustix::{AsRawFd, RawFd},
    std::os::fd::{AsFd, BorrowedFd},
};

/// A wrapper around a `Read` which implements `ReadTerminal` but isn't ever
/// a terminal.
#[derive(Debug)]
pub struct NeverTerminalReader<Inner: Read> {
    inner: Inner,
}

impl<Inner: Read> NeverTerminalReader<Inner> {
    /// Wrap a `NeverTerminalReader` around the given stream.
    #[inline]
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
impl<Inner: Read + AsRawFd> AsRawFd for NeverTerminalReader<Inner> {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

#[cfg(not(windows))]
impl<Inner: Read + AsFd> AsFd for NeverTerminalReader<Inner> {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.inner.as_fd()
    }
}

#[cfg(windows)]
impl<Inner: Read + AsRawHandleOrSocket> AsRawHandleOrSocket for NeverTerminalReader<Inner> {
    #[inline]
    fn as_raw_handle_or_socket(&self) -> RawHandleOrSocket {
        self.inner.as_raw_handle_or_socket()
    }
}

#[cfg(windows)]
impl<Inner: Read + AsHandleOrSocket> AsHandleOrSocket for NeverTerminalReader<Inner> {
    #[inline]
    fn as_handle_or_socket(&self) -> BorrowedHandleOrSocket {
        self.inner.as_handle_or_socket()
    }
}

impl<Inner: Read> Terminal for NeverTerminalReader<Inner> {}

impl<Inner: Read> ReadTerminal for NeverTerminalReader<Inner> {
    #[inline]
    fn is_line_by_line(&self) -> bool {
        false
    }

    #[inline]
    fn is_input_terminal(&self) -> bool {
        false
    }
}

impl<Inner: Read> Read for NeverTerminalReader<Inner> {
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
