//! The `TerminalDuplex` struct.

use crate::config::{detect_read_write_config, ReadConfig, WriteConfig};
use crate::{DuplexTerminal, ReadTerminal, Terminal, TerminalColorSupport, WriteTerminal};
use duplex::{Duplex, HalfDuplex};
use std::fmt;
use std::io::{self, IoSlice, IoSliceMut, Read, Write};
#[cfg(windows)]
use unsafe_io::os::windows::{
    AsRawReadWriteHandleOrSocket, AsReadWriteHandleOrSocket, BorrowedHandleOrSocket,
    RawHandleOrSocket,
};
use unsafe_io::AsReadWriteGrip;
#[cfg(not(windows))]
use {
    io_lifetimes::BorrowedFd,
    unsafe_io::os::rsix::{AsRawReadWriteFd, AsReadWriteFd, RawFd},
};

/// A wrapper around a `Read` + `Write` which adds minimal terminal support.
#[derive(Debug)]
pub struct TerminalDuplexer<Inner: Duplex> {
    inner: Inner,
    read_config: Option<ReadConfig>,
    write_config: Option<WriteConfig>,
}

impl<Inner: Duplex + AsReadWriteGrip> TerminalDuplexer<Inner> {
    /// Wrap a `TerminalDuplex` around the given stream, autodetecting
    /// terminal properties using its `AsGrip` implementation.
    pub fn with_handle<'a>(inner: Inner) -> Self {
        let (read_config, write_config) = detect_read_write_config(&inner);
        Self {
            inner,
            read_config,
            write_config,
        }
    }
}

impl<Inner: Duplex + Read + Write> TerminalDuplexer<Inner> {
    /// Wrap a `TerminalReader` around the given stream, using
    /// conservative terminal properties.
    pub fn generic(inner: Inner) -> Self {
        Self {
            inner,
            read_config: None,
            write_config: None,
        }
    }

    /// Consume `self` and return the inner stream.
    #[inline]
    pub fn into_inner(self) -> Inner {
        self.inner
    }

    fn reset(&mut self) {
        if self.is_output_terminal() {
            self.write("\x1b[!p\r\x1b[K".as_bytes()).ok();
        }
    }
}

#[cfg(not(windows))]
impl<Inner: Duplex + AsRawReadWriteFd> AsRawReadWriteFd for TerminalDuplexer<Inner> {
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
impl<Inner: Duplex + AsReadWriteFd> AsReadWriteFd for TerminalDuplexer<Inner> {
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
    for TerminalDuplexer<Inner>
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
    for TerminalDuplexer<Inner>
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

impl<Inner: Duplex> Terminal for TerminalDuplexer<Inner> {}

impl<Inner: Duplex + Read + Write> ReadTerminal for TerminalDuplexer<Inner> {
    fn is_line_by_line(&self) -> bool {
        self.read_config.as_ref().map_or(false, |c| c.line_by_line)
    }

    fn is_input_terminal(&self) -> bool {
        self.read_config.is_some()
    }
}

impl<Inner: Duplex + Read + Write> WriteTerminal for TerminalDuplexer<Inner> {
    fn color_support(&self) -> TerminalColorSupport {
        self.write_config
            .as_ref()
            .map_or_else(Default::default, |c| c.color_support)
    }

    fn color_preference(&self) -> bool {
        self.write_config
            .as_ref()
            .map_or(false, |c| c.color_preference)
    }

    fn is_output_terminal(&self) -> bool {
        self.write_config.is_some()
    }
}

impl<Inner: Duplex + HalfDuplex> DuplexTerminal for TerminalDuplexer<Inner> {}

impl<Inner: Duplex + Read + Write> Read for TerminalDuplexer<Inner> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.inner.read(buf) {
            Ok(0) if !buf.is_empty() => {
                self.reset();
                Ok(0)
            }
            Ok(n) => Ok(n),
            Err(e) => Err(e),
        }
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut]) -> io::Result<usize> {
        match self.inner.read_vectored(bufs) {
            Ok(0) if bufs.iter().any(|b| !b.is_empty()) => {
                self.reset();
                Ok(0)
            }
            Ok(n) => Ok(n),
            Err(e) => Err(e),
        }
    }

    #[cfg(can_vector)]
    #[inline]
    fn is_read_vectored(&self) -> bool {
        self.inner.is_read_vectored()
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let n = self.inner.read_to_end(buf)?;
        self.reset();
        Ok(n)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        let n = self.inner.read_to_string(buf)?;
        self.reset();
        Ok(n)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        match self.inner.read_exact(buf) {
            Ok(()) => Ok(()),
            Err(e) => {
                if e.kind() == io::ErrorKind::UnexpectedEof {
                    self.reset();
                }
                Err(e)
            }
        }
    }
}

impl<Inner: Duplex + Read + Write> Write for TerminalDuplexer<Inner> {
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

impl<Inner: Duplex> Duplex for TerminalDuplexer<Inner> {}
