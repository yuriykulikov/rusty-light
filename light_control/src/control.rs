use no_std_compat::cell::Cell;

use crate::bsp::led::{Led, PWM_MAX};
use crate::bsp::pin::Pin;
use crate::bsp::rgb::{BLUE, GREEN, RED, Rgb};
use crate::control::Action::CheckButtons;
use crate::edt::EDT;

/// Control logic evaluates button states and changes the light intensity
pub struct LightControl<'a, P: Pin> {
    pub plus_pin: P,
    pub minus_pin: P,
    pub led: &'a dyn Led,
    pub rgb: &'a dyn Rgb,
    pub edt: &'a EDT<Action>,
    pub led_level: Cell<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq, Copy)]
pub enum Action {
    Blink { color: u8, blinks: u32 },
    CheckButtons { prev_plus: u32, prev_minus: u32 },
}

pub const DELAY_CHECK_BUTTONS: u32 = 75;
pub const DELAY_BLINK: u32 = 250;

pub const PWM_STEPS: &'static [u32] = &[0, 1, 4, 9, 16, 25, 36, 49, 64, 81, PWM_MAX];

impl<'a, P: Pin> LightControl<'a, P> {
    pub fn process_message(&self, action: Action) {
        match action {
            Action::CheckButtons { prev_plus, prev_minus } => self.check_buttons(prev_plus, prev_minus),
            Action::Blink { color, blinks } => self.blink_led(color, blinks),
        }
    }

    fn blink_led(&self, color: u8, blinks: u32) {
        if blinks > 0 {
            let rgb = self.rgb.get_rgb();
            self.rgb.set_rgb(rgb ^ color);
            self.edt.schedule(DELAY_BLINK, Action::Blink { color, blinks: blinks - 1 });
        }
    }

    pub fn check_buttons(&self, prev_plus: u32, prev_minus: u32) {
        if self.plus_pin.is_down() {
            self.edt.schedule(DELAY_CHECK_BUTTONS, CheckButtons { prev_plus: prev_plus + 1, prev_minus });
        } else if self.minus_pin.is_down() {
            self.edt.schedule(DELAY_CHECK_BUTTONS, CheckButtons { prev_plus, prev_minus: prev_minus + 1 });
        } else {
            if prev_plus > 0 {
                self.on_plus_clicked();
            }
            if prev_minus > 0 {
                self.on_minus_clicked()
            }
            self.edt.schedule(DELAY_CHECK_BUTTONS, CheckButtons { prev_plus: 0, prev_minus: 0 });
        }
    }

    pub fn start(&self) {
        self.check_buttons(0, 0);
    }

    fn on_plus_clicked(&self) {
        self.increment_led_level(1);
        self.rgb.set_rgb(GREEN);
        self.remove_blinks();
        self.edt.schedule(DELAY_BLINK, Action::Blink { color: GREEN, blinks: 5 });
    }

    fn on_minus_clicked(&self) {
        self.decrement_led_level(1);

        if self.led.get_pwm() == 0 {
            self.rgb.set_rgb(BLUE);
            self.remove_blinks();
            self.edt.schedule(1000, Action::Blink { color: BLUE, blinks: 1 });
        } else {
            self.rgb.set_rgb(RED);
            self.remove_blinks();
            self.edt.schedule(DELAY_BLINK, Action::Blink { color: RED, blinks: 5 });
        }
    }

    fn remove_blinks(&self) {
        self.edt.remove(|action| {
            match action {
                Action::Blink { color: _, blinks: _ } => true,
                _ => false
            }
        });
    }

    fn increment_led_level(&self, inc: usize) {
        self.change_led_level(inc, true);
    }
    fn decrement_led_level(&self, dec: usize) {
        self.change_led_level(dec, false);
    }
    fn change_led_level(&self, change: usize, inc: bool) {
        let max_level = PWM_STEPS.len() - 1;
        let current = self.led_level.get();
        if inc && current == max_level { return; }
        if !inc && current == 0 { return; }

        let new_level = if inc { current + change } else { current - change };
        self.led_level.set(new_level);
        self.led.set_pwm(PWM_STEPS[new_level]);
    }
}


