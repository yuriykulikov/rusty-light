use crate::InputPin;
use light_control::bsp::pin::Pin;

pub struct PullUpButton<INPUT: InputPin> {
    pub(crate) pin: INPUT,
}

impl<INPUT: InputPin> Pin for PullUpButton<INPUT> {
    fn is_down(&self) -> bool {
        return self.pin.is_low().unwrap_or(false);
    }
}
