//! Kernel /dev/uinput device implementation.

use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::thread::sleep;
use std::time::Duration;

/// Linux evdev event types (<linux/input-event-codes.h>)
pub const EV_SYN: u16 = 0x00;
pub const EV_KEY: u16 = 0x01;

/// Linux evdev key codes (<linux/input-event-codes.h>)
pub const KEY_BACKSPACE: u16 = 14;

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
    #[allow(dead_code)]
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
            self.send_key_event(KEY_BACKSPACE, true);
            self.send_key_event(KEY_BACKSPACE, false);
            if delay_ms > 0 {
                sleep(Duration::from_millis(delay_ms));
            }
        }
    }

    /// Commits a sequence of characters directly.
    pub fn emit_string(&mut self, _text: &str) {
        // Direct commit Unicode glyphs via uinput or XKB keysym mapping
    }

    fn send_key_event(&mut self, code: u16, press: bool) {
        let _ev = libc::input_event {
            time: libc::timeval { tv_sec: 0, tv_usec: 0 },
            type_: EV_KEY,
            code,
            value: if press { 1 } else { 0 },
        };
        // write _ev into device_file
    }
}
