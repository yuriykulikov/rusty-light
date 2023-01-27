use core::cell::RefCell;

use stm_hal::analog::adc::Adc;
use stm_hal::hal::adc::Channel;

use light_control::bsp::adc::Sensors;
use light_control::perceived_light_math::current_ma;
use light_control::voltage_to_temp::voltage_to_temp;

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
    fn battery_voltage(&self, high_percentage: u32, low_percentage: u32) -> u32 {
        let mut voltage = self.measure() as u64;
        let samples = 100;
        for _ in 0..samples {
            voltage += self.measure() as u64;
        }

        voltage = voltage / samples;

        let v_bat: u32 = (voltage as u32) * (self.r_pull_up + self.r_pull_down) / self.r_pull_down;

        if v_bat < 5000 {
            0
        } else {
            (current_ma(v_bat, 850, low_percentage) + current_ma(v_bat, 1000, high_percentage))
                * 320
                / 1000
                + v_bat
        }
    }

    fn temp(&self) -> i32 {
        let measured = self
            .adc
            .borrow_mut()
            .read_voltage(&mut *self.vin_temp.borrow_mut())
            .unwrap() as u32;
        voltage_to_temp(measured)
    }
}

impl<V, T> AdcSensors<V, T>
where
    T: Channel<Adc, ID = u8>,
    V: Channel<Adc, ID = u8>,
{
    fn measure(&self) -> u16 {
        self.adc
            .borrow_mut()
            .read_voltage(&mut *self.vin_pin.borrow_mut())
            .unwrap()
    }
}
