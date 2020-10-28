#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use light_control::control::{LightControl, PWM_STEPS, DELAY_CHECK_BUTTONS};
    use light_control::edt::EDT;
    use light_control::bsp::led::{PWM_MAX, Led};
    use light_control::bsp::pin::Pin;
    use light_control::bsp::rgb::Rgb;

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
        let rgb = TestRgb { rgb: Cell::new(0) };
        let edt = EDT::create();
        let light_control = LightControl {
            plus_pin: TestPin { is_down: &plus_pin },
            minus_pin: TestPin { is_down: &minus_pin },
            led: &led,
            edt: &edt,
            rgb: &rgb,
            led_level: Cell::new(0),
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

    /// Led which resides in memory, for simulation or testing
    pub struct TestRgb {
        rgb: Cell<u8>
    }

    impl Rgb for TestRgb {
        fn set_rgb(&self, rgb: u8) {
            self.rgb.set(rgb);
        }
        fn get_rgb(&self) -> u8 {
            return self.rgb.get();
        }
    }
}