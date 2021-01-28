use crate::TerminalColorSupport;
use duplex::Duplex;
#[cfg(not(windows))]
use std::mem::MaybeUninit;
#[cfg(unix)]
use std::os::unix::io::{AsRawFd, RawFd};
#[cfg(target_os = "wasi")]
use std::os::wasi::io::{AsRawFd, RawFd};
use unsafe_io::{AsUnsafeHandle, AsUnsafeReadWriteHandle};
#[cfg(windows)]
use {
    std::os::windows::io::AsRawHandle,
    unsafe_io::{AsRawHandleOrSocket, RawHandleOrSocket},
};

#[derive(Debug)]
pub(crate) struct ReadConfig {
    pub(crate) line_by_line: bool,
}

#[derive(Default, Debug)]
pub(crate) struct WriteConfig {
    pub(crate) color_support: TerminalColorSupport,
    pub(crate) color_preference: bool,
}

impl Default for ReadConfig {
    fn default() -> Self {
        Self {
            line_by_line: false,
        }
    }
}

pub(crate) fn detect_read_config<AUH: AsUnsafeHandle>(handle: &AUH) -> Option<ReadConfig> {
    #[cfg(not(windows))]
    unsafe {
        _detect_read_config(handle.as_unsafe_handle().as_raw_fd())
    }

    #[cfg(windows)]
    unsafe {
        _detect_read_config(handle.as_unsafe_handle().as_raw_handle_or_socket())
    }
}

pub(crate) fn detect_write_config<AUH: AsUnsafeHandle>(handle: &AUH) -> Option<WriteConfig> {
    #[cfg(not(windows))]
    unsafe {
        _detect_write_config(handle.as_unsafe_handle().as_raw_fd())
    }

    #[cfg(windows)]
    unsafe {
        _detect_write_config(handle.as_unsafe_handle().as_raw_handle_or_socket())
    }
}

pub(crate) fn detect_read_write_config<AUH: Duplex + AsUnsafeReadWriteHandle>(
    handle: &AUH,
) -> (Option<ReadConfig>, Option<WriteConfig>) {
    let unsafe_read_handle = handle.as_unsafe_read_handle();
    let unsafe_write_handle = handle.as_unsafe_read_handle();

    let (read_config, write_config) = if unsafe { unsafe_write_handle.eq(unsafe_read_handle) } {
        #[cfg(not(windows))]
        let read_config = unsafe { _detect_read_config(unsafe_read_handle.as_raw_fd()) };

        #[cfg(windows)]
        let read_config =
            unsafe { _detect_read_config(unsafe_read_handle.as_raw_handle_or_socket()) };

        let write_config = if read_config.is_some() {
            #[cfg(not(windows))]
            unsafe {
                Some(_detect_write_config_isatty(unsafe_write_handle.as_raw_fd()))
            }

            #[cfg(windows)]
            unsafe {
                let color_preference = if unsafe_write_handle
                    .eq(std::io::stdout().as_unsafe_handle())
                    || unsafe_write_handle.eq(std::io::stderr().as_unsafe_handle())
                {
                    detect_stdio_color_preference()
                } else {
                    false
                };
                Some(_detect_write_config_isatty(
                    unsafe_write_handle.as_raw_handle_or_socket(),
                    color_preference,
                ))
            }
        } else {
            #[cfg(not(windows))]
            unsafe {
                _detect_write_config(unsafe_write_handle.as_raw_fd())
            }

            #[cfg(windows)]
            unsafe {
                _detect_write_config(unsafe_write_handle.as_raw_handle_or_socket())
            }
        };

        (read_config, write_config)
    } else {
        #[cfg(not(windows))]
        let read_config = unsafe { _detect_read_config(unsafe_read_handle.as_raw_fd()) };

        #[cfg(windows)]
        let read_config =
            unsafe { _detect_read_config(unsafe_read_handle.as_raw_handle_or_socket()) };

        #[cfg(not(windows))]
        let write_config = unsafe { _detect_write_config(unsafe_write_handle.as_raw_fd()) };

        #[cfg(windows)]
        let write_config =
            unsafe { _detect_write_config(unsafe_write_handle.as_raw_handle_or_socket()) };

        (read_config, write_config)
    };

    (read_config, write_config)
}

#[cfg(not(windows))]
unsafe fn _detect_read_config(raw_fd: RawFd) -> Option<ReadConfig> {
    let mut termios = MaybeUninit::<libc::termios>::uninit();

    if libc::tcgetattr(raw_fd, termios.as_mut_ptr()) == 0 {
        Some(ReadConfig {
            line_by_line: (termios.assume_init().c_lflag & libc::ICANON) == libc::ICANON,
        })
    } else {
        // `tcgetattr` fails when it's not reading from a terminal.
        None
    }
}

#[cfg(windows)]
unsafe fn _detect_read_config(raw_handle_or_socket: RawHandleOrSocket) -> Option<ReadConfig> {
    let isatty = match raw_handle_or_socket.as_raw_handle() {
        Some(handle) => {
            if handle == std::io::stdin().as_raw_handle() {
                atty::is(atty::Stream::Stdin)
            } else {
                false
            }
        }
        None => false,
    };

    if isatty {
        Some(ReadConfig {
            // TODO: Is there a way to do this on Windows?
            line_by_line: false,
        })
    } else {
        None
    }
}

#[cfg(not(windows))]
unsafe fn _detect_write_config(raw_fd: RawFd) -> Option<WriteConfig> {
    if posish::io::isatty(&raw_fd) {
        Some(_detect_write_config_isatty(raw_fd))
    } else {
        None
    }
}

#[cfg(not(windows))]
unsafe fn _detect_write_config_isatty(raw_fd: RawFd) -> WriteConfig {
    let (color_support, color_preference) = if raw_fd == std::io::stdout().as_raw_fd() {
        let info = terminfo::Database::from_env().unwrap();
        let color_support = info.get::<terminfo::capability::MaxColors>().map_or_else(
            TerminalColorSupport::default,
            |num| {
                let num: i32 = num.into();
                // TODO: Detect TrueColor support
                match num {
                    -1 => TerminalColorSupport::Monochrome,
                    8 => TerminalColorSupport::Classic8,
                    256 => TerminalColorSupport::ColorCube256,
                    _ => panic!("Unrecognized color count {}", num),
                }
            },
        );

        let color_preference = detect_stdio_color_preference();

        (color_support, color_preference)
    } else {
        (TerminalColorSupport::default(), false)
    };

    WriteConfig {
        color_support,
        color_preference,
    }
}

#[cfg(windows)]
unsafe fn _detect_write_config(raw_handle_or_socket: RawHandleOrSocket) -> Option<WriteConfig> {
    let (isatty, color_preference) = match raw_handle_or_socket.as_raw_handle() {
        Some(handle) => {
            if handle == std::io::stdout().as_raw_handle() {
                (
                    atty::is(atty::Stream::Stdout),
                    detect_stdio_color_preference(),
                )
            } else if handle == std::io::stderr().as_raw_handle() {
                (
                    atty::is(atty::Stream::Stderr),
                    detect_stdio_color_preference(),
                )
            } else {
                (false, false)
            }
        }
        None => (false, false),
    };

    if isatty {
        Some(_detect_write_config_isatty(
            raw_handle_or_socket,
            color_preference,
        ))
    } else {
        None
    }
}

#[cfg(windows)]
unsafe fn _detect_write_config_isatty(
    _raw_handle_or_socket: RawHandleOrSocket,
    color_preference: bool,
) -> WriteConfig {
    // Windows supports the 24-bit escape sequence but doesn't actually
    // display the full color range.
    // https://docs.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences#extended-colors
    let color_support = TerminalColorSupport::Classic8;

    WriteConfig {
        color_support,
        color_preference,
    }
}

fn detect_stdio_color_preference() -> bool {
    std::env::var_os("NO_COLOR").is_none()
}
