use core::cell::RefCell;

use stm_hal::analog::adc::Adc;
use stm_hal::hal::adc::Channel;

use light_control::bsp::adc::Sensors;

pub struct AdcSensors<V: Channel<Adc, ID = u8>, T: Channel<Adc, ID = u8>> {
    pub adc: RefCell<Adc>,
    pub vin_pin: RefCell<V>,
    pub r_pull_up: u32,
    pub r_pull_down: u32,
    pub vin_temp: RefCell<T>,
}

impl<V, T> Sensors for AdcSensors<V, T>
where
    V: Channel<Adc, ID = u8>,
    T: Channel<Adc, ID = u8>,
{
    fn battery_voltage(&self) -> u32 {
        let measured = self
            .adc
            .borrow_mut()
            .read_voltage(&mut *self.vin_pin.borrow_mut())
            .unwrap() as u32;
        measured * (self.r_pull_up + self.r_pull_down) / self.r_pull_down
    }

    fn temp(&self) -> u32 {
        let measured = self
            .adc
            .borrow_mut()
            .read_voltage(&mut *self.vin_temp.borrow_mut())
            .unwrap() as u32;
        measured
    }
}
