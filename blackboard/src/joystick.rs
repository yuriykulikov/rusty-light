use core::cell::RefCell;

use stm_hal::analog::adc::Adc;

use light_control::bsp::joystick::Joystick;

use crate::hal::adc::Channel;

pub struct AdcJoystick<PinV: Channel<Adc, ID = u8>, PinH: Channel<Adc, ID = u8>> {
    adc_pin_v: RefCell<PinV>,
    adc_pin_h: RefCell<PinH>,
    adc: RefCell<Adc>,
}

impl<PinV: Channel<Adc, ID = u8>, PinH: Channel<Adc, ID = u8>> AdcJoystick<PinV, PinH> {
    pub(crate) fn create(adc_pin_v: PinV, adc_pin_h: PinH, adc: Adc) -> Self {
        AdcJoystick {
            adc_pin_v: RefCell::new(adc_pin_v),
            adc_pin_h: RefCell::new(adc_pin_h),
            adc: RefCell::new(adc),
        }
    }
}

impl<PinV: Channel<Adc, ID = u8>, PinH: Channel<Adc, ID = u8>> Joystick
    for AdcJoystick<PinV, PinH>
{
    fn read(&self) -> (i32, i32) {
        let uh_mv = self
            .adc
            .borrow_mut()
            .read_voltage(&mut *self.adc_pin_h.borrow_mut())
            .expect("adc read failed") as u32;
        let uv_mv = self
            .adc
            .borrow_mut()
            .read_voltage(&mut *self.adc_pin_v.borrow_mut())
            .expect("adc read failed") as u32;
        let x = ((uh_mv as i32) - 1660) / (1660 / 50);
        let y = ((uv_mv as i32) - 1660) / (1660 / 50);
        (x, y)
    }
}
