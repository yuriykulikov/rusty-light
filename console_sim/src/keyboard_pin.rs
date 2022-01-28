use keyboard_query;
use keyboard_query::{DeviceQuery, DeviceState};

use light_control::bsp::joystick::Joystick;
use light_control::bsp::pin::Pin;

pub struct KeyboardPin {
    device_state: DeviceState,
    key_code: u16,
}

impl KeyboardPin {
    /// Factory function to create a [KeyboardPin]
    pub fn create(key_code: u16) -> KeyboardPin {
        let device_state = DeviceState::new();
        return KeyboardPin {
            device_state,
            key_code,
        };
    }
}

impl Pin for KeyboardPin {
    /// returns true is pin is tied to the ground
    fn is_down(&self) -> bool {
        let keys = &self.device_state.get_keys();
        return keys.contains(&self.key_code);
    }
}

pub struct DummyJoystick {
    pub(crate) left: KeyboardPin,
    pub(crate) right: KeyboardPin,
    pub(crate) up: KeyboardPin,
    pub(crate) down: KeyboardPin,
}

impl Joystick for DummyJoystick {
    fn read(&self) -> (i32, i32) {
        if self.left.is_down() {
            (-50, 0)
        } else if self.right.is_down() {
            (50, 0)
        } else if self.up.is_down() {
            (0, 50)
        } else if self.down.is_down() {
            (0, -50)
        } else {
            (0, 0)
        }
    }
}
