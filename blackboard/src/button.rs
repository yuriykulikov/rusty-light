use light_control::bsp::pin::Pin;

use crate::InputPin;

pub struct PullUpButton<INPUT: InputPin> {
    pub(crate) pin: INPUT,
}

impl<INPUT: InputPin> Pin for PullUpButton<INPUT> {
    fn is_down(&self) -> bool {
        return self.pin.is_low().unwrap_or(false);
    }
}

pub struct NopButton {}

impl Pin for NopButton {
    fn is_down(&self) -> bool {
        false
    }
}
