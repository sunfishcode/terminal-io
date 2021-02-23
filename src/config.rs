use crate::TerminalColorSupport;
use duplex::Duplex;
use unsafe_io::{AsUnsafeHandle, AsUnsafeReadWriteHandle, ReadHalf, WriteHalf};
#[cfg(not(windows))]
use {std::mem::MaybeUninit, unsafe_io::os::posish::AsRawFd};
#[cfg(windows)]
use {std::os::windows::io::AsRawHandle, unsafe_io::os::windows::AsRawHandleOrSocket};

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

pub(crate) fn detect_read_write_config<AUH: Duplex + AsUnsafeReadWriteHandle>(
    handle: &AUH,
) -> (Option<ReadConfig>, Option<WriteConfig>) {
    let read_half = ReadHalf(handle);
    let write_half = WriteHalf(handle);

    let (read_config, write_config) = if write_half.eq_handle(&read_half) {
        let read_config = detect_read_config(&read_half);

        let write_config = if read_config.is_some() {
            #[cfg(not(windows))]
            {
                Some(detect_write_config_isatty(&write_half))
            }

            #[cfg(windows)]
            {
                let color_preference = if write_half.eq_handle(&std::io::stdout())
                    || write_half.eq_handle(&std::io::stderr())
                {
                    detect_stdio_color_preference()
                } else {
                    false
                };
                Some(detect_write_config_isatty(&write_half, color_preference))
            }
        } else {
            detect_write_config(&write_half)
        };

        (read_config, write_config)
    } else {
        let read_config = detect_read_config(&read_half);
        let write_config = detect_write_config(&write_half);
        (read_config, write_config)
    };

    (read_config, write_config)
}

#[cfg(not(windows))]
pub(crate) fn detect_read_config<AUH: AsUnsafeHandle>(handle: &AUH) -> Option<ReadConfig> {
    let mut termios = MaybeUninit::<libc::termios>::uninit();

    unsafe {
        if libc::tcgetattr(handle.as_unsafe_handle().as_raw_fd(), termios.as_mut_ptr()) == 0 {
            Some(ReadConfig {
                line_by_line: (termios.assume_init().c_lflag & libc::ICANON) == libc::ICANON,
            })
        } else {
            // `tcgetattr` fails when it's not reading from a terminal.
            None
        }
    }
}

#[cfg(windows)]
pub(crate) fn detect_read_config<AUH: AsUnsafeHandle>(handle: &AUH) -> Option<ReadConfig> {
    let isatty = match handle
        .as_unsafe_handle()
        .as_raw_handle_or_socket()
        .as_raw_handle()
    {
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
pub(crate) fn detect_write_config<AUH: AsUnsafeHandle>(handle: &AUH) -> Option<WriteConfig> {
    if posish::io::isatty(handle) {
        Some(detect_write_config_isatty(handle))
    } else {
        None
    }
}

#[cfg(not(windows))]
fn detect_write_config_isatty<AUH: AsUnsafeHandle>(handle: &AUH) -> WriteConfig {
    let (color_support, color_preference) = if handle.eq_handle(&std::io::stdout()) {
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
pub(crate) fn detect_write_config<AUH: AsUnsafeHandle>(handle: &AUH) -> Option<WriteConfig> {
    let (isatty, color_preference) = match handle
        .as_unsafe_handle()
        .as_raw_handle_or_socket()
        .as_raw_handle()
    {
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
        Some(detect_write_config_isatty(handle, color_preference))
    } else {
        None
    }
}

#[cfg(windows)]
fn detect_write_config_isatty<AUH: AsUnsafeHandle>(
    _handle: &AUH,
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
