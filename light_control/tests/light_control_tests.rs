#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::mem::size_of_val;

    use light_control::bsp::led::{Led, MAX};
    use light_control::bsp::pin::Pin;
    use light_control::bsp::rgb::Rgb;
    use light_control::control::{
        Action, LightControl, ANIM_DURATION, BUTTON_CHECK_PERIOD, MAX_POWER_LEVEL, POWER_LEVELS_LOW,
    };
    use light_control::edt::EDT;

    #[test]
    fn edt_queue_size_is_below_1kb() {
        let edt = EDT::<Action>::create();
        assert!(size_of_val(&edt.queue) < 500);
    }

    #[test]
    fn starting_brightness() {
        with_bench(&|advance_time, _buttons, power_level| {
            // startup animation
            advance_time(2000);
            assert_eq!(power_level.get(), low(3));
        });
    }

    #[test]
    fn plus_click_increases_brightness() {
        with_bench(&|_advance_time, buttons, power_level| {
            buttons.click_plus();
            assert_eq!(power_level.get(), low(4));
        });
    }

    #[test]
    fn plus_lick_increases_brightness_until_max_reached() {
        with_bench(&|_advance_time, buttons, power_level| {
            for _ in 0..3 {
                buttons.click_plus();
            }
            assert_eq!(power_level.get(), low(MAX_POWER_LEVEL));
        });
    }

    #[test]
    fn minus_decreases_brightness() {
        with_bench(&|_advance_time, buttons, power_level| {
            // given brightness is max
            buttons.click_plus();
            assert_eq!(power_level.get(), low(4));

            buttons.click_minus();
            assert_eq!(power_level.get(), low(3));
            buttons.click_minus();
            assert_eq!(power_level.get(), low(2));
            buttons.click_minus();
            assert_eq!(power_level.get(), low(1));
        });
    }

    #[test]
    fn minus_decreases_brightness_until_min_reached() {
        with_bench(&|_advance_time, buttons, power_level| {
            buttons.click_minus();
            buttons.click_minus();
            buttons.click_minus();
            buttons.click_minus();
            assert_eq!(power_level.get(), low(1));
        });
    }

    #[test]
    fn when_off_minus_button_clicks_switches_on() {
        with_bench(&|_advance_time, buttons, power_level| {
            buttons.long_click_plus();
            buttons.click_minus();
            assert_eq!(power_level.get(), low(2));
        });
    }

    /// Clicks here are below the longclick threshold, but they are longer than usual clicks
    #[test]
    fn longer_clicks_have_effect_when_released() {
        with_bench(&|advance_time, buttons, power_level| {
            // startup animation
            advance_time(2000);
            assert_eq!(power_level.get(), low(3));
            buttons.press_minus();
            advance_time(700);
            buttons.release_minus();
            advance_time(100 + ANIM_DURATION);
            assert_eq!(power_level.get(), low(2));

            buttons.press_plus();
            advance_time(700);
            buttons.release_plus();
            advance_time(100 + ANIM_DURATION);
            assert_eq!(power_level.get(), low(3));

            buttons.press_plus();
            advance_time(700);
            buttons.release_plus();
            advance_time(100 + ANIM_DURATION);
            assert_eq!(power_level.get(), low(4));
        });
    }

    fn low(i: usize) -> u32 {
        assert!(
            i < POWER_LEVELS_LOW.len(),
            "Test level exceeds available levels: {}",
            i
        );
        POWER_LEVELS_LOW[i] as u32
    }

    fn with_bench(block: &dyn Fn(&dyn Fn(u32), Buttons, &Cell<u32>)) {
        let plus_pin = Cell::new(false);
        let minus_pin = Cell::new(false);
        let toggle_pin = Cell::new(false);
        let power_level = Cell::new(0);
        let power_level_high = Cell::new(0);
        let led = TestLed {
            power_output: &power_level,
        };
        let led_high = TestLed {
            power_output: &power_level_high,
        };
        let rgb = TestRgb { rgb: Cell::new(0) };
        let edt = EDT::create();
        let light_control = LightControl::new(
            TestPin { is_down: &plus_pin },
            TestPin {
                is_down: &minus_pin,
            },
            TestPin {
                is_down: &toggle_pin,
            },
            &led,
            &led_high,
            &rgb,
            &edt,
        );
        light_control.start();
        light_control.jump_start();
        edt.advance_time_by(1000, &|msg| {
            light_control.process_message(msg);
        });

        let prev_led = Cell::new(0);
        let advance_time = |time: u32| {
            edt.advance_time_by(time, &|msg| {
                light_control.process_message(msg);
                if prev_led.get() != led.get() {
                    render_flashlight_state(led.get(), rgb.get_rgb());
                }
                prev_led.set(led.get());
            });
        };

        block(
            &advance_time,
            Buttons {
                plus_pin: &plus_pin,
                minus_pin: &minus_pin,
                toggle_pin: &toggle_pin,
                advance_time: &advance_time,
            },
            &power_level,
        );
    }

    fn render_flashlight_state(power_level: u32, _rgb: u8) {
        let mut led_str = String::new();
        for _ in 0..power_level {
            led_str.push('*');
        }
        for _ in 0..(MAX - power_level) {
            led_str.push(' ');
        }
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
        pub power_output: &'a Cell<u32>,
    }

    impl<'a> Led for TestLed<'a> {
        fn set(&self, power_level: u32) {
            self.power_output.set(power_level);
        }

        fn get(&self) -> u32 {
            return self.power_output.get();
        }
    }

    pub struct Buttons<'a> {
        plus_pin: &'a Cell<bool>,
        minus_pin: &'a Cell<bool>,
        toggle_pin: &'a Cell<bool>,
        advance_time: &'a dyn Fn(u32),
    }

    impl<'a> Buttons<'a> {
        fn press_plus(&self) {
            self.plus_pin.set(true);
        }
        fn release_plus(&self) {
            self.plus_pin.set(false);
        }
        fn click_plus(&self) {
            (self.advance_time)(BUTTON_CHECK_PERIOD);
            self.press_plus();
            (self.advance_time)(BUTTON_CHECK_PERIOD);
            self.release_plus();
            (self.advance_time)(BUTTON_CHECK_PERIOD);
            (self.advance_time)(ANIM_DURATION);
        }
        fn press_minus(&self) {
            self.minus_pin.set(true);
        }
        fn release_minus(&self) {
            self.minus_pin.set(false);
        }
        fn click_minus(&self) {
            (self.advance_time)(BUTTON_CHECK_PERIOD);
            self.press_minus();
            (self.advance_time)(BUTTON_CHECK_PERIOD);
            self.release_minus();
            (self.advance_time)(BUTTON_CHECK_PERIOD);
            (self.advance_time)(ANIM_DURATION);
        }
        fn long_click_plus(&self) {
            (self.advance_time)(BUTTON_CHECK_PERIOD);
            self.press_plus();
            (self.advance_time)(1500);
            self.release_plus();
            (self.advance_time)(BUTTON_CHECK_PERIOD);
            (self.advance_time)(ANIM_DURATION);
        }
    }

    /// Led which resides in memory, for simulation or testing
    pub struct TestRgb {
        rgb: Cell<u8>,
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
