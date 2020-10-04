/// Power LED of the flashlight. [pwn] represents the duty cycle.
pub trait Led {
    fn set_pwm(&mut self, pwm: u8);
    fn get_pwm(&self) -> u8;
}

/// Led which resides in memory, for simulation or testing
pub struct DummyLed {
    pwm: u8
}

impl DummyLed {
    /// Factory function to create a dummy LED
    pub fn create(pwm: u8) -> Box<dyn Led> {
        return Box::new(DummyLed { pwm });
    }
}

/// Higher level factory function to create a dummy LED
/// TODO which one is idiomatic?
pub fn led_create_dummy(pwm: u8) -> Box<dyn Led> {
    return Box::new(DummyLed { pwm });
}

impl Led for DummyLed {
    fn set_pwm(&mut self, pwm: u8) {
        self.pwm = pwm
    }

    fn get_pwm(&self) -> u8 {
        return self.pwm;
    }
}

