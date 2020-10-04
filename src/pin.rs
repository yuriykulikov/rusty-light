extern crate keyboard_query;

use keyboard_query::{DeviceQuery, DeviceState};

/// A pin (of a button) which may be down (tied to the ground) or up (floating pin)
pub trait Pin {
    fn is_down(&self) -> bool;
}

pub struct KeyboardPin {
    device_state: DeviceState,
    key_code: u16,
}

impl KeyboardPin {
    /// Factory function to create a [KeyboardPin]
    pub fn create(key_code: u16) -> KeyboardPin {
        let device_state = DeviceState::new();
        return KeyboardPin { device_state, key_code };
    }

    /// Factory function to create a [KeyboardPin]
    /// TODO is it ok to use Box with factory functions?
    pub fn create_boxed(key_code: u16) -> Box<dyn Pin> {
        let device_state = DeviceState::new();
        return Box::new(KeyboardPin { device_state, key_code });
    }
}

impl Pin for KeyboardPin {
    /// returns true is pin is tied to the ground
    fn is_down(&self) -> bool {
        let keys = &self.device_state.get_keys();
        return keys.contains(&self.key_code);
    }
}
