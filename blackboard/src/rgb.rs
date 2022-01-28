use core::cell::{Cell, RefCell};
use core::convert::Infallible;

use light_control::bsp::rgb::Rgb;

use crate::hal::digital::v2::OutputPin;

pub struct GpioRgb<OUTPUT: OutputPin<Error = Infallible>> {
    pub(crate) pin: RefCell<OUTPUT>,
    pub(crate) state: Cell<u8>,
}

impl<OUTPUT: OutputPin<Error = Infallible>> Rgb for GpioRgb<OUTPUT> {
    fn set_rgb(&self, rgb: u8) {
        self.state.set(rgb);
        if rgb == 0 {
            self.pin.borrow_mut().set_low().unwrap();
        } else {
            self.pin.borrow_mut().set_high().unwrap();
        }
    }

    fn get_rgb(&self) -> u8 {
        return self.state.get();
    }
}
