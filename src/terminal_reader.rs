//! The `TerminalReader` struct.

use crate::config::{detect_read_config, ReadConfig};
use crate::{ReadTerminal, Terminal};
use io_extras::grip::AsGrip;
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

/// A wrapper around a `Read` which adds minimal terminal support.
#[derive(Debug)]
pub struct TerminalReader<Inner: Read> {
    inner: Inner,
    read_config: Option<ReadConfig>,
}

impl<Inner: Read + AsGrip> TerminalReader<Inner> {
    /// Wrap a `TerminalReader` around the given stream, autodetecting
    /// terminal properties using its `AsGrip` implementation.
    #[inline]
    pub fn with_handle(inner: Inner) -> Self {
        let read_config = detect_read_config(&inner);
        Self { inner, read_config }
    }
}

impl<Inner: Read> TerminalReader<Inner> {
    /// Wrap a `TerminalReader` around the given stream, using
    /// conservative terminal properties.
    #[inline]
    pub fn generic(inner: Inner) -> Self {
        Self {
            inner,
            read_config: None,
        }
    }

    /// Consume `self` and return the inner stream.
    #[inline]
    pub fn into_inner(self) -> Inner {
        self.inner
    }
}

#[cfg(not(windows))]
impl<Inner: Read + AsRawFd> AsRawFd for TerminalReader<Inner> {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

#[cfg(not(windows))]
impl<Inner: Read + AsFd> AsFd for TerminalReader<Inner> {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.inner.as_fd()
    }
}

#[cfg(windows)]
impl<Inner: Read + AsRawHandleOrSocket> AsRawHandleOrSocket for TerminalReader<Inner> {
    #[inline]
    fn as_raw_handle_or_socket(&self) -> RawHandleOrSocket {
        self.inner.as_raw_handle_or_socket()
    }
}

#[cfg(windows)]
impl<Inner: Read + AsHandleOrSocket> AsHandleOrSocket for TerminalReader<Inner> {
    #[inline]
    fn as_handle_or_socket(&self) -> BorrowedHandleOrSocket<'_> {
        self.inner.as_handle_or_socket()
    }
}

impl<Inner: Read> Terminal for TerminalReader<Inner> {}

impl<Inner: Read> ReadTerminal for TerminalReader<Inner> {
    #[inline]
    fn is_line_by_line(&self) -> bool {
        self.read_config.as_ref().map_or(false, |c| c.line_by_line)
    }

    #[inline]
    fn is_input_terminal(&self) -> bool {
        self.read_config.is_some()
    }
}

impl<Inner: Read> Read for TerminalReader<Inner> {
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
