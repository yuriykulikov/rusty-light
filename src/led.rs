use std::cell::Cell;

/// Power LED of the flashlight. [pwn] represents the duty cycle.
pub trait Led {
    fn set_pwm(&self, pwm: u8);
    fn get_pwm(&self) -> u8;
    fn modify(&self, f: &dyn Fn(u8) -> u8);
}

/// Led which resides in memory, for simulation or testing
pub struct DummyLed {
    pwm: Cell<u8>
}

impl DummyLed {
    /// Factory function to create a dummy LED
    pub fn create(pwm: u8) -> Self {
        return DummyLed { pwm: Cell::new(pwm) };
    }
}

impl Led for DummyLed {
    fn set_pwm(&self, pwm: u8) {
        self.pwm.set(pwm);
    }

    fn get_pwm(&self) -> u8 {
        return self.pwm.get();
    }

    fn modify(&self, f: &dyn Fn(u8) -> u8) {
        let prev = self.get_pwm();
        let value = f(prev);
        self.set_pwm(value)
    }
}

