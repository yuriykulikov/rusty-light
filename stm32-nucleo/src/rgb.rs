use core::cell::{Cell, RefCell};
use core::convert::Infallible;

use light_control::bsp::rgb::{Rgb, BLUE, GREEN, RED};

use crate::hal::digital::v2::OutputPin;

pub struct GpioRgb<R, G, B>
where
    R: OutputPin<Error = Infallible>,
    G: OutputPin<Error = Infallible>,
    B: OutputPin<Error = Infallible>,
{
    pub(crate) r: RefCell<R>,
    pub(crate) g: RefCell<G>,
    pub(crate) b: RefCell<B>,
    pub(crate) state: Cell<u8>,
}

impl<R, G, B> Rgb for GpioRgb<R, G, B>
where
    R: OutputPin<Error = Infallible>,
    G: OutputPin<Error = Infallible>,
    B: OutputPin<Error = Infallible>,
{
    fn set_rgb(&self, rgb: u8) {
        self.state.set(rgb);
        if rgb & RED > 0 {
            self.r.borrow_mut().set_low().unwrap()
        } else {
            self.r.borrow_mut().set_high().unwrap()
        }
        if rgb & GREEN > 0 {
            self.g.borrow_mut().set_low().unwrap()
        } else {
            self.g.borrow_mut().set_high().unwrap()
        }
        if rgb & BLUE > 0 {
            self.b.borrow_mut().set_low().unwrap()
        } else {
            self.b.borrow_mut().set_high().unwrap()
        }
    }

    fn get_rgb(&self) -> u8 {
        return self.state.get();
    }
}
