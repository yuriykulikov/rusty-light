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
pub const LONG_CLICK_THRESHOLD: u32 = 1000 / DELAY_CHECK_BUTTONS;
pub const DELAY_BLINK: u32 = 100;

pub const PWM_STEPS: &'static [u32] = &[0, 1, 4, 9, 16, 25, 36, 49, 64, 81, PWM_MAX];
const PWM_MAX_LEVEL: usize = PWM_STEPS.len() - 1;

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
            if prev_plus == LONG_CLICK_THRESHOLD {
                self.on_long_clicked();
            }
        } else {
            if (1..LONG_CLICK_THRESHOLD).contains(&prev_plus) {
                self.on_plus_clicked();
            }
        }

        if self.minus_pin.is_down() {
            if prev_minus == LONG_CLICK_THRESHOLD {
                self.on_long_clicked();
            }
        } else {
            if (1..LONG_CLICK_THRESHOLD).contains(&prev_minus) {
                self.on_minus_clicked();
            }
        }

        let next_plus = if self.plus_pin.is_down() { prev_plus + 1 } else { 0 };
        let next_minus = if self.minus_pin.is_down() { prev_minus + 1 } else { 0 };

        self.edt.schedule(DELAY_CHECK_BUTTONS, CheckButtons { prev_plus: next_plus, prev_minus: next_minus });
    }

    pub fn start(&self) {
        self.check_buttons(0, 0);
    }

    fn on_plus_clicked(&self) {
        if self.led_level.get() > 0 && self.led_level.get() < PWM_MAX_LEVEL {
            self.increment_led_level(1);
            self.blink(GREEN, 5, DELAY_BLINK);
        } else {
            self.indicate_nop();
        }
    }

    fn on_minus_clicked(&self) {
        if self.led_level.get() > 1 {
            self.decrement_led_level(1);
            self.blink(RED, 5, DELAY_BLINK);
        } else {
            self.indicate_nop();
        }
    }

    fn on_long_clicked(&self) {
        if self.led_level.get() == 0 {
            self.led_level.set(3);
            self.led.set_pwm(PWM_STEPS[3]);
            self.blink(GREEN, 9, DELAY_BLINK / 2);
        } else {
            self.led_level.set(0);
            self.led.set_pwm(PWM_STEPS[0]);
            self.blink(RED, 9, DELAY_BLINK / 2);
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

    fn blink(&self, color: u8, times: u32, period: u32) {
        self.rgb.set_rgb(color);
        self.remove_blinks();
        self.edt.schedule(period, Action::Blink { color, blinks: times });
    }

    fn indicate_nop(&self) {
        self.blink(BLUE, 1, 500);
    }

    fn increment_led_level(&self, inc: usize) {
        self.change_led_level(inc, true);
    }

    fn decrement_led_level(&self, dec: usize) {
        self.change_led_level(dec, false);
    }

    fn change_led_level(&self, change: usize, inc: bool) {
        let current = self.led_level.get();
        if inc && current == PWM_MAX_LEVEL { return; }
        if !inc && current == 0 { return; }

        let new_level = if inc { current + change } else { current - change };
        self.led_level.set(new_level);
        self.led.set_pwm(PWM_STEPS[new_level]);
    }
}


