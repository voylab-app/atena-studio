//! Process creation helpers to prevent unwanted console windows on Windows GUI applications.

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Windows process creation flag that prevents creating a console window (0x08000000).
pub const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Extension trait to configure `std::process::Command` and `tokio::process::Command`
/// to run silently without popping console / terminal windows on Windows.
pub trait SilentCommand {
    fn silent_windows(&mut self) -> &mut Self;
}

impl SilentCommand for std::process::Command {
    #[inline]
    fn silent_windows(&mut self) -> &mut Self {
        #[cfg(windows)]
        {
            self.creation_flags(CREATE_NO_WINDOW);
        }
        self
    }
}

impl SilentCommand for tokio::process::Command {
    #[inline]
    fn silent_windows(&mut self) -> &mut Self {
        #[cfg(windows)]
        {
            self.creation_flags(CREATE_NO_WINDOW);
        }
        self
    }
}

/// Creates a `std::process::Command` pre-configured to suppress console windows on Windows.
pub fn silent_command<P: AsRef<std::ffi::OsStr>>(program: P) -> std::process::Command {
    let mut cmd = std::process::Command::new(program);
    cmd.silent_windows();
    cmd
}

/// Creates a `tokio::process::Command` pre-configured to suppress console windows on Windows.
pub fn silent_tokio_command<P: AsRef<std::ffi::OsStr>>(program: P) -> tokio::process::Command {
    let mut cmd = tokio::process::Command::new(program);
    cmd.silent_windows();
    cmd
}
