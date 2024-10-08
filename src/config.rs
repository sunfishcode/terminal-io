use crate::TerminalColorSupport;
use duplex::Duplex;
use io_extras::grip::{AsGrip, AsRawGrip, AsReadWriteGrip};
use io_extras::read_write::{ReadHalf, WriteHalf};
#[cfg(windows)]
use is_terminal::IsTerminal;
#[cfg(windows)]
use {io_extras::os::windows::AsHandleOrSocket, std::os::windows::io::AsRawHandle};

#[derive(Default, Debug)]
pub(crate) struct ReadConfig {
    pub(crate) line_by_line: bool,
}

#[derive(Default, Debug)]
pub(crate) struct WriteConfig {
    pub(crate) color_support: TerminalColorSupport,
    pub(crate) color_preference: bool,
}

pub(crate) fn detect_read_write_config<Grip: Duplex + AsReadWriteGrip>(
    handle: &Grip,
) -> (Option<ReadConfig>, Option<WriteConfig>) {
    let read_half = ReadHalf::new(handle);
    let write_half = WriteHalf::new(handle);

    let (read_config, write_config) =
        if write_half.as_grip().as_raw_grip() == read_half.as_grip().as_raw_grip() {
            let read_config = detect_read_config(&read_half);

            let write_config = if read_config.is_some() {
                #[cfg(not(windows))]
                {
                    Some(detect_write_config_isatty(&write_half))
                }

                #[cfg(windows)]
                {
                    let color_preference = if write_half.as_grip().as_raw_grip()
                        == std::io::stdout().as_grip().as_raw_grip()
                        || write_half.as_grip().as_raw_grip()
                            == std::io::stderr().as_grip().as_raw_grip()
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
pub(crate) fn detect_read_config<Grip: AsGrip>(handle: &Grip) -> Option<ReadConfig> {
    match rustix::termios::tcgetattr(handle) {
        Ok(termios) => Some(ReadConfig {
            line_by_line: termios
                .local_modes
                .contains(rustix::termios::LocalModes::ICANON),
        }),
        Err(_) => {
            // `tcgetattr` fails when it's not reading from a terminal.
            None
        }
    }
}

#[cfg(windows)]
pub(crate) fn detect_read_config<Grip: AsGrip>(handle: &Grip) -> Option<ReadConfig> {
    let isatty = match handle.as_grip().as_handle_or_socket().as_handle() {
        Some(handle) => handle.is_terminal(),
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
pub(crate) fn detect_write_config<Grip: AsGrip>(handle: &Grip) -> Option<WriteConfig> {
    if rustix::termios::isatty(handle) {
        Some(detect_write_config_isatty(handle))
    } else {
        None
    }
}

#[cfg(not(windows))]
fn detect_write_config_isatty<Grip: AsGrip>(handle: &Grip) -> WriteConfig {
    let (color_support, color_preference) =
        if handle.as_grip().as_raw_grip() == std::io::stdout().as_grip().as_raw_grip() {
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
pub(crate) fn detect_write_config<Grip: AsGrip>(grip: &Grip) -> Option<WriteConfig> {
    match grip.as_grip().as_handle_or_socket().as_handle() {
        Some(handle) if handle.is_terminal() => {
            let color_preference = if handle.as_raw_handle() == std::io::stdout().as_raw_handle() {
                detect_stdio_color_preference()
            } else if handle.as_raw_handle() == std::io::stderr().as_raw_handle() {
                detect_stdio_color_preference()
            } else {
                false
            };

            Some(detect_write_config_isatty(grip, color_preference))
        }
        _ => None,
    }
}

#[cfg(windows)]
fn detect_write_config_isatty<Grip: AsGrip>(_handle: &Grip, color_preference: bool) -> WriteConfig {
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
