use crate::led::Led;
use crate::pin::Pin;

/// Control logic evaluates button states and changes the light intensity
pub struct LightControl<'a, P: Pin> {
    pub plus_pin: P,
    pub minus_pin: P,
    pub led: &'a dyn Led,
}

impl <'a, P: Pin> LightControl<'a, P> {
    pub fn tick(&self) {
        if self.plus_pin.is_down() {
            self.led.modify(&|current: u8| {
                if current < 8 { current + 1 } else { current }
            });
        }

        if self.minus_pin.is_down() {
            self.led.modify(&|current: u8| {
                if current > 0 { current - 1 } else { current }
            });
        }
    }
}