//! Kernel /dev/uinput device implementation.

use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::thread::sleep;
use std::time::Duration;

/// Uinput device error.
#[derive(Debug)]
pub enum UinputError {
    /// Cannot open /dev/uinput (permission denied or device missing)
    DeviceAccessError(std::io::Error),
    /// ioctl configuration failure
    IoctlError(&'static str),
}

/// Linux virtual keyboard device controller.
pub struct UinputKeyboard {
    device_file: File,
}

impl UinputKeyboard {
    /// Opens /dev/uinput and registers a virtual keyboard device.
    pub fn open() -> Result<Self, UinputError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open("/dev/uinput")
            .map_err(UinputError::DeviceAccessError)?;

        // Setup evdev keyboard keys
        // (EV_KEY, KEY_BACKSPACE, KEY_A..KEY_Z, etc.)

        Ok(Self { device_file: file })
    }

    /// Emits N consecutive backspace events with Sentinel timing fence.
    pub fn emit_backspaces(&mut self, count: usize, delay_ms: u64) {
        for _ in 0..count {
            self.send_key_event(libc::KEY_BACKSPACE as u16, true);
            self.send_key_event(libc::KEY_BACKSPACE as u16, false);
            if delay_ms > 0 {
                sleep(Duration::from_millis(delay_ms));
            }
        }
    }

    /// Commits a sequence of characters directly.
    pub fn emit_string(&mut self, text: &str) {
        // Direct commit Unicode glyphs via uinput or XKB keysym mapping
    }

    fn send_key_event(&mut self, code: u16, press: bool) {
        let ev = libc::input_event {
            time: libc::timeval { tv_sec: 0, tv_usec: 0 },
            type_: libc::EV_KEY as u16,
            code,
            value: if press { 1 } else { 0 },
        };
        // write ev into device_file
    }
}
