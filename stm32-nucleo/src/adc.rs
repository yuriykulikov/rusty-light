use core::cell::RefCell;

use stm_hal::analog::adc::Adc;
use stm_hal::hal::adc::Channel;

use light_control::bsp::adc::Sensors;

pub struct AdcSensors<T: Channel<Adc, ID = u8>> {
    pub adc: RefCell<Adc>,
    pub vin_pin: RefCell<T>,
    pub r_pull_up: u32,
    pub r_pull_down: u32,
}

impl<T: Channel<Adc, ID = u8>> Sensors for AdcSensors<T> {
    fn battery_voltage(&self) -> u32 {
        let measured = self
            .adc
            .borrow_mut()
            .read_voltage(&mut *self.vin_pin.borrow_mut())
            .expect("adc read failed") as u32;
        measured * (self.r_pull_up + self.r_pull_down) / self.r_pull_down
    }
}
