use std::cell::Cell;

use light_control::bsp::led::Led;

/// Led which resides in memory, for simulation or testing
pub struct DummyLed {
    power_output: Cell<u32>,
}

impl DummyLed {
    /// Factory function to create a dummy LED
    pub fn create(pwm: u32) -> Self {
        return DummyLed {
            power_output: Cell::new(pwm),
        };
    }
}

impl Led for DummyLed {
    fn set(&self, pwm: u32) {
        self.power_output.set(pwm);
    }

    fn get(&self) -> u32 {
        return self.power_output.get();
    }
}
