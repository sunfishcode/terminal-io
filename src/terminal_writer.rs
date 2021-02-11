//! The `TerminalWriter` struct.

use crate::{
    config::{detect_write_config, WriteConfig},
    Terminal, TerminalColorSupport, WriteTerminal,
};
#[cfg(unix)]
use std::os::unix::io::{AsRawFd, RawFd};
#[cfg(target_os = "wasi")]
use std::os::wasi::io::{AsRawFd, RawFd};
use std::{
    fmt,
    io::{self, IoSlice, Write},
};
use unsafe_io::AsUnsafeHandle;
#[cfg(windows)]
use unsafe_io::{AsRawHandleOrSocket, RawHandleOrSocket};

/// A wrapper around a `Write` which adds minimal terminal support.
#[derive(Debug)]
pub struct TerminalWriter<Inner: Write> {
    inner: Inner,
    write_config: Option<WriteConfig>,
}

impl<Inner: Write + AsUnsafeHandle> TerminalWriter<Inner> {
    /// Wrap a `TerminalWriter` around the given stream, autodetecting
    /// terminal properties using its `AsUnsafeHandle` implementation.
    pub fn with_handle(inner: Inner) -> Self {
        let write_config = detect_write_config(&inner);
        Self {
            inner,
            write_config,
        }
    }

    /// Wrap a `TerminalWriter` around the given stream, using the given
    /// terminal properties.
    pub fn from(
        inner: Inner,
        is_terminal: bool,
        color_support: TerminalColorSupport,
        color_preference: bool,
    ) -> Self {
        Self {
            inner,
            write_config: if is_terminal {
                Some(WriteConfig {
                    color_support,
                    color_preference,
                })
            } else {
                None
            },
        }
    }
}

impl<Inner: Write> TerminalWriter<Inner> {
    /// Wrap a `TerminalWriter` around the given stream, using
    /// conservative terminal properties.
    pub fn generic(inner: Inner) -> Self {
        Self {
            inner,
            write_config: None,
        }
    }

    /// Consume `self` and return the inner stream.
    #[inline]
    pub fn into_inner(self) -> Inner {
        self.inner
    }
}

#[cfg(not(windows))]
impl<Inner: Write + AsRawFd> AsRawFd for TerminalWriter<Inner> {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

#[cfg(windows)]
impl<Inner: Write + AsRawHandleOrSocket> AsRawHandleOrSocket for TerminalWriter<Inner> {
    #[inline]
    fn as_raw_handle_or_socket(&self) -> RawHandleOrSocket {
        self.inner.as_raw_handle_or_socket()
    }
}

impl<Inner: Write> Terminal for TerminalWriter<Inner> {}

impl<Inner: Write> WriteTerminal for TerminalWriter<Inner> {
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

impl<Inner: Write> Write for TerminalWriter<Inner> {
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
