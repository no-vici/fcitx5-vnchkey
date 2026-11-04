//! Safe Linux /dev/uinput Virtual Keyboard Device.

pub mod device;

pub use device::{UinputKeyboard, UinputError, EV_KEY, KEY_BACKSPACE};
