use std::cell::Cell;

pub const PWM_MAX: u32 = 100;

/// Power LED of the flashlight. [pwn] represents the duty cycle.
pub trait Led {
    fn set_pwm(&self, pwm: u32);
    fn get_pwm(&self) -> u32;
    fn modify(&self, f: &dyn Fn(u32) -> u32);
}

/// Led which resides in memory, for simulation or testing
pub struct DummyLed {
    pwm: Cell<u32>
}

impl DummyLed {
    /// Factory function to create a dummy LED
    pub fn create(pwm: u32) -> Self {
        return DummyLed { pwm: Cell::new(pwm) };
    }
}

impl Led for DummyLed {
    fn set_pwm(&self, pwm: u32) {
        self.pwm.set(pwm);
    }

    fn get_pwm(&self) -> u32 {
        return self.pwm.get();
    }

    fn modify(&self, f: &dyn Fn(u32) -> u32) {
        let prev = self.get_pwm();
        let value = f(prev);
        self.set_pwm(value)
    }
}

