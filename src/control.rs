use crate::control::Action::CheckButtons;
use crate::event_loop::EDT;
use crate::led::{Led, PWM_MAX};
use crate::pin::Pin;
use crate::rgb::{BLUE, GREEN, RED, Rgb};

/// Control logic evaluates button states and changes the light intensity
pub struct LightControl<'a, P: Pin> {
    pub plus_pin: P,
    pub minus_pin: P,
    pub led: &'a dyn Led,
    pub rgb: &'a dyn Rgb,
    pub edt: &'a EDT<Action>,
}

pub enum Action {
    Blink { color: u8, blinks: u32 },
    CheckButtons { prev_plus: u32, prev_minus: u32 },
}

const DELAY_CHECK_BUTTONS: u32 = 75;
const DELAY_BLINK: u32 = 250;

const PWM_STEPS: &'static [u32] = &[0, 1, 4, 9, 16, 25, 36, 49, 64, 81, PWM_MAX];

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
        self.led.modify(&|current: u32| {
            // Regarding && see
            // https://stackoverflow.com/questions/43828013/why-is-being-used-in-closure-arguments
            *PWM_STEPS
                .iter()
                // first higher
                .find(|&&brightness| { brightness > current })
                .unwrap_or(&PWM_MAX)
        });
        self.rgb.set_rgb(GREEN);
        self.remove_blinks();
        self.edt.schedule(DELAY_BLINK, Action::Blink { color: GREEN, blinks: 5 });
    }

    fn on_minus_clicked(&self) {
        self.led.modify(&|current: u32| {
            *PWM_STEPS
                .iter()
                // last lower
                .filter(|&&brightness| { brightness < current })
                .last()
                .unwrap_or(&0)
        });

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
}


#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use crate::control::{DELAY_CHECK_BUTTONS, LightControl, PWM_STEPS};
    use crate::event_loop::EDT;
    use crate::led::{Led, PWM_MAX};
    use crate::pin::Pin;
    use crate::rgb::{DummyRgb, Rgb};

    #[test]
    fn button_clicks_change_brightness() {
        with_bench(&|_advance_time, buttons, pwm| {
            for _ in 0..10 {
                buttons.click_plus();
            }
            assert_eq!(pwm.get(), 100);

            for _ in 0..10 {
                buttons.click_minus();
            }
            assert_eq!(pwm.get(), 0);
        });
    }

    #[test]
    fn clicks_can_be_spread_over_time() {
        with_bench(&|advance_time, buttons, pwm| {
            for _ in 0..5 {
                buttons.click_plus();
                advance_time(1000);
            }
            assert_eq!(pwm.get(), 25);

            for _ in 0..5 {
                buttons.click_minus();
                advance_time(1000);
            }
            assert_eq!(pwm.get(), 0);
        });
    }

    #[test]
    fn long_clicks_have_effect_when_released() {
        with_bench(&|advance_time, buttons, pwm| {
            assert_eq!(pwm.get(), PWM_STEPS[0]);
            for i in 1..5 {
                buttons.press_plus();
                advance_time(10000);
                buttons.release_plus();
                advance_time(100);
                assert_eq!(pwm.get(), PWM_STEPS[i]);
            }
        });
    }

    fn with_bench(block: &dyn Fn(&dyn Fn(u32), Buttons, &Cell<u32>)) {
        let plus_pin = Cell::new(false);
        let minus_pin = Cell::new(false);
        let pwm = Cell::new(0);
        let led = TestLed { pwm: &pwm };
        let rgb = DummyRgb::create();
        let edt = EDT::create();
        let light_control = LightControl {
            plus_pin: TestPin { is_down: &plus_pin },
            minus_pin: TestPin { is_down: &minus_pin },
            led: &led,
            edt: &edt,
            rgb: &rgb,
        };
        light_control.start();

        let advance_time = |time: u32| {
            edt.process_events(time, &|action| {
                light_control.process_message(action);
                render_flashlight_state(led.get_pwm(), rgb.get_rgb());
            });
        };

        block(
            &advance_time,
            Buttons { plus_pin: &plus_pin, minus_pin: &minus_pin, advance_time: &advance_time },
            &pwm,
        );
    }

    fn render_flashlight_state(pwm: u32, rgb: u8) {
        let mut led_str = String::new();
        for _ in 0..pwm { led_str.push('*'); }
        for _ in 0..(PWM_MAX - pwm) { led_str.push(' '); }
        println!("  [{}]  [{}]", led_str, rgb);
    }

    struct TestPin<'a> {
        is_down: &'a Cell<bool>,
    }

    impl<'a> Pin for TestPin<'a> {
        /// returns true is pin is tied to the ground
        fn is_down(&self) -> bool {
            return self.is_down.get();
        }
    }

    /// Led which resides in memory, for simulation or testing
    pub struct TestLed<'a> {
        pub pwm: &'a Cell<u32>
    }

    impl<'a> Led for TestLed<'a> {
        fn set_pwm(&self, pwm: u32) {
            self.pwm.set(pwm);
        }

        fn get_pwm(&self) -> u32 {
            return self.pwm.get();
        }

        fn modify(&self, f: &dyn Fn(u32) -> u32) {
            let prev = self.get_pwm();
            let value = f(prev);
            self.set_pwm(value)
        }
    }

    pub struct Buttons<'a> {
        plus_pin: &'a Cell<bool>,
        minus_pin: &'a Cell<bool>,
        advance_time: &'a dyn Fn(u32),
    }

    impl<'a> Buttons<'a> {
        fn press_plus(&self) { self.plus_pin.set(true); }
        fn release_plus(&self) { self.plus_pin.set(false); }
        fn click_plus(&self) {
            (self.advance_time)(DELAY_CHECK_BUTTONS);
            self.press_plus();
            (self.advance_time)(DELAY_CHECK_BUTTONS);
            self.release_plus();
            (self.advance_time)(DELAY_CHECK_BUTTONS);
        }
        fn press_minus(&self) { self.minus_pin.set(true); }
        fn release_minus(&self) { self.minus_pin.set(false); }
        fn click_minus(&self) {
            (self.advance_time)(DELAY_CHECK_BUTTONS);
            self.press_minus();
            (self.advance_time)(DELAY_CHECK_BUTTONS);
            self.release_minus();
            (self.advance_time)(DELAY_CHECK_BUTTONS);
        }
    }
}