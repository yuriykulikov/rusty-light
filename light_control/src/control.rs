use no_std_compat::cell::Cell;

use crate::bsp::led::{Led, MAX};
use crate::bsp::pin::Pin;
use crate::bsp::rgb::{BLUE, GREEN, RED, Rgb};
use crate::control::Action::{CheckButtons, SetPwm};
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
    SetPwm { power_level: u32 },
}

pub const DELAY_CHECK_BUTTONS: u32 = 50;
pub const LONG_CLICK_THRESHOLD: u32 = 1000 / DELAY_CHECK_BUTTONS;
pub const DELAY_BLINK: u32 = 100;

pub const POWER_LEVELS: &'static [u32] = &[0, 7, 20, 40, 60, 80, MAX];
const PWM_POWER_LEVEL: usize = POWER_LEVELS.len() - 1;
pub const ANIM_DURATION: u32 = 250;
const ANIM_SIZE: usize = 20;
const ANIM_STEP: u32 = ANIM_DURATION / ANIM_SIZE as u32;

impl<'a, P: Pin> LightControl<'a, P> {
    pub fn process_message(&self, action: Action) {
        match action {
            Action::CheckButtons { prev_plus, prev_minus } => self.check_buttons(prev_plus, prev_minus),
            Action::Blink { color, blinks } => self.blink_led(color, blinks),
            Action::SetPwm { power_level: goal } => self.set_power_level(goal),
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
        if self.led_level.get() == 0 {
            self.set_led_level_with_animation(4, elastic_steps);
            self.blink(GREEN, 9, DELAY_BLINK / 2);
        } else if self.led_level.get() < PWM_POWER_LEVEL {
            self.increment_led_level(1);
            self.blink(GREEN, 5, DELAY_BLINK);
        } else {
            self.indicate_nop();
        }
    }

    fn on_minus_clicked(&self) {
        if self.led_level.get() == 0 {
            self.set_led_level_with_animation(2, linear_sine_exp_steps);
            self.blink(GREEN, 9, DELAY_BLINK / 2);
        } else if self.led_level.get() > 1 {
            self.decrement_led_level(1);
            self.blink(RED, 5, DELAY_BLINK);
        } else {
            self.indicate_nop();
        }
    }

    fn on_long_clicked(&self) {
        if self.led_level.get() == 0 {
            self.set_led_level_with_animation(1, linear_steps);
            self.blink(BLUE, 3, DELAY_BLINK / 2);
        } else {
            self.set_led_level_with_animation(0, linear_steps);
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
        if inc && current == PWM_POWER_LEVEL { return; }
        if !inc && current == 0 { return; }

        let new_level = if inc { current + change } else { current - change };

        self.set_led_level_with_animation(new_level, linear_steps);
    }

    fn set_led_level_with_animation(&self, new_level: usize, animation: fn(u32, u32) -> [u32; ANIM_SIZE]) {
        self.led_level.set(new_level);

        // animation
        self.remove_set_power_level_messages();

        let current_power_level = self.led.get();
        let goal = POWER_LEVELS[new_level];

        let steps = animation(current_power_level, goal);
        for i in 0..ANIM_SIZE {
            self.edt.schedule(ANIM_STEP * i as u32, SetPwm { power_level: steps[i] });
        }
    }

    fn remove_set_power_level_messages(&self) {
        self.edt.remove(|msg| {
            match msg {
                Action::SetPwm { power_level: _ } => true,
                _ => false
            }
        });
    }

    fn set_power_level(&self, power_level: u32) {
        self.led.set(power_level);
    }
}

fn linear_steps(from: u32, to: u32) -> [u32; ANIM_SIZE] {
    let mut x = [1234; ANIM_SIZE];

    let diff = to as i32 - from as i32;

    for i in 0..ANIM_SIZE as i32 {
        let next_value = from as i32 + (diff * (i + 1) / ANIM_SIZE as i32);
        debug_assert!(next_value >= 0);
        x[i as usize] = next_value as u32;
    }
    return x;
}

fn linear_sine_exp_steps(from: u32, to: u32) -> [u32; ANIM_SIZE] {
    debug_assert_eq!(from, 0);
    debug_assert!(to == 20 || to == 40);
    return if to == 40 {
        [4, 8, 12, 16, 20, 24, 28, 32, 36, 40, 44, 45, 44, 42, 40, 39, 38, 38, 39, 40]
    } else {
        [2, 4, 6, 8, 10, 12, 14, 16, 18, 21, 24, 25, 24, 22, 20, 19, 18, 18, 19, 20]
    };
}

fn elastic_steps(from: u32, to: u32) -> [u32; ANIM_SIZE] {
    debug_assert_eq!(from, 0);
    debug_assert!(to == 40 || to == 60);
    return if to == 40 {
        [7, 18, 29, 39, 48, 53, 55, 54, 52, 48, 44, 40, 38, 36, 35, 35, 36, 37, 38, 40]
    } else {
        [11, 27, 44, 59, 71, 79, 82, 81, 78, 72, 66, 61, 56, 53, 52, 52, 54, 55, 58, 60]
    };
}

#[cfg(test)]
mod tests {
    use crate::control::linear_steps;

    #[test]
    fn linear_step_up() {
        let steps = linear_steps(0, 20);
        assert_eq!(steps, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]);
    }

    #[test]
    fn linear_step_up_big() {
        let steps = linear_steps(0, 40);
        assert_eq!(steps, [2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 34, 36, 38, 40]);
    }


    #[test]
    fn linear_step_down() {
        let steps = linear_steps(60, 80);
        assert_eq!(steps, [61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80]);
    }

    #[test]
    fn linear_step_down_big() {
        let steps = linear_steps(80, 0);
        assert_eq!(steps, [76, 72, 68, 64, 60, 56, 52, 48, 44, 40, 36, 32, 28, 24, 20, 16, 12, 8, 4, 0]);
    }
}




