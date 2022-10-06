use crate::hal;
use core::cell::{Cell, RefCell};
use light_control::bsp::led::Led;
use light_control::perceived_light_math::fill_pwm_duty_cycle_values;

pub struct PwmLed<PWM: hal::PwmPin<Duty = u16>> {
    duties: [u16; 101],
    pwm_ch: RefCell<PWM>,
    state: Cell<u32>,
}

impl<PWM: hal::PwmPin<Duty = u16>> PwmLed<PWM> {
    pub(crate) fn create(pwm_ch: PWM) -> Self {
        let max = pwm_ch.get_max_duty();

        let mut led = PwmLed {
            duties: [0; 101],
            pwm_ch: RefCell::new(pwm_ch),
            state: Cell::new(0),
        };

        led.pwm_ch.borrow_mut().set_duty(0);
        led.pwm_ch.borrow_mut().enable();

        fill_pwm_duty_cycle_values(&mut led.duties, 0, max);

        return led;
    }
}

impl<PWM: hal::PwmPin<Duty = u16>> Led for PwmLed<PWM> {
    fn set(&self, pwm: u32) {
        self.state.set(pwm);
        let duty_cycle = self.duties[pwm as usize];
        self.pwm_ch.borrow_mut().set_duty(duty_cycle);
    }

    fn get(&self) -> u32 {
        return self.state.get();
    }
}
