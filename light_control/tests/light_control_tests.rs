#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use light_control::bsp::led::{Led, MAX};
    use light_control::bsp::pin::Pin;
    use light_control::bsp::rgb::Rgb;
    use light_control::control::{ANIM_DURATION, DELAY_CHECK_BUTTONS, LightControl, POWER_LEVELS};
    use light_control::edt::EDT;
    use light_control::bsp::joystick::Joystick;

    #[test]
    fn plus_button_clicks_switch_on() {
        with_bench(&|_advance_time, buttons, power_level| {
            buttons.click_plus();
            assert_eq!(power_level.get(), 60);
        });
    }

    #[test]
    fn minus_button_clicks_switch_on() {
        with_bench(&|_advance_time, buttons, power_level| {
            buttons.click_minus();
            assert_eq!(power_level.get(), 20);
        });
    }

    #[test]
    fn button_long_click_switches_on_low_mode() {
        with_bench(&|_advance_time, buttons, power_level| {
            buttons.long_click_plus();
            assert_eq!(power_level.get(), 7);
        });
    }

    #[test]
    fn button_clicks_change_brightness() {
        with_bench(&|_advance_time, buttons, power_level| {
            buttons.click_minus();

            for _ in 0..2 {
                buttons.click_plus();
            }
            assert_eq!(power_level.get(), 60);

            for _ in 0..2 {
                buttons.click_minus();
            }
            assert_eq!(power_level.get(), 20);
        });
    }

    #[test]
    fn brightness_can_be_changed_up_to_100() {
        with_bench(&|_advance_time, buttons, power_level| {
            buttons.long_click_plus();
            for _ in 0..10 {
                buttons.click_plus();
            }
            assert_eq!(power_level.get(), 100);
        });
    }

    #[test]
    fn brightness_can_be_changed_down_to_1() {
        with_bench(&|_advance_time, buttons, power_level| {
            buttons.long_click_plus();
            for _ in 0..10 {
                buttons.click_minus();
            }
            assert_eq!(power_level.get(), 7);
        });
    }

    #[test]
    fn clicks_can_be_spread_over_time() {
        with_bench(&|advance_time, buttons, power_level| {
            buttons.click_minus();

            for _ in 0..3 {
                buttons.click_plus();
                advance_time(1000);
            }

            assert_eq!(power_level.get(), 80);

            for _ in 0..3 {
                buttons.click_minus();
                advance_time(1000);
            }

            assert_eq!(power_level.get(), 20);
        });
    }

    #[test]
    fn long_clicks_have_effect_when_released() {
        with_bench(&|advance_time, buttons, power_level| {
            buttons.click_minus();
            assert_eq!(power_level.get(), POWER_LEVELS[2]);
            for i in 3..6 {
                buttons.press_plus();
                advance_time(700);
                buttons.release_plus();
                advance_time(100 + ANIM_DURATION);
                assert_eq!(power_level.get(), POWER_LEVELS[i]);
            }
        });
    }

    fn with_bench(block: &dyn Fn(&dyn Fn(u32), Buttons, &Cell<u32>)) {
        let plus_pin = Cell::new(false);
        let minus_pin = Cell::new(false);
        let power_level = Cell::new(0);
        let led = TestLed { power_output: &power_level };
        let rgb = TestRgb { rgb: Cell::new(0) };
        let edt = EDT::create();
        let light_control = LightControl {
            plus_pin: TestPin { is_down: &plus_pin },
            minus_pin: TestPin { is_down: &minus_pin },
            joystick: TestJoystick {},
            led: &led,
            edt: &edt,
            rgb: &rgb,
            led_level: Cell::new(0),
            furthest_stick_position: Cell::new((0, 0)),
        };
        light_control.start();

        let advance_time = |time: u32| {
            edt.advance_time_by(time, &|msg| {
                light_control.process_message(msg);
                render_flashlight_state(led.get(), rgb.get_rgb());
            });
        };

        block(
            &advance_time,
            Buttons { plus_pin: &plus_pin, minus_pin: &minus_pin, advance_time: &advance_time },
            &power_level,
        );
    }

    fn render_flashlight_state(power_level: u32, rgb: u8) {
        let mut led_str = String::new();
        for _ in 0..power_level { led_str.push('*'); }
        for _ in 0..(MAX - power_level) { led_str.push(' '); }
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
            (self.advance_time)(ANIM_DURATION);
        }
        fn press_minus(&self) { self.minus_pin.set(true); }
        fn release_minus(&self) { self.minus_pin.set(false); }
        fn click_minus(&self) {
            (self.advance_time)(DELAY_CHECK_BUTTONS);
            self.press_minus();
            (self.advance_time)(DELAY_CHECK_BUTTONS);
            self.release_minus();
            (self.advance_time)(DELAY_CHECK_BUTTONS);
            (self.advance_time)(ANIM_DURATION);
        }
        fn long_click_plus(&self) {
            (self.advance_time)(DELAY_CHECK_BUTTONS);
            self.press_plus();
            (self.advance_time)(1500);
            self.release_plus();
            (self.advance_time)(DELAY_CHECK_BUTTONS);
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

    struct TestJoystick {}

    impl Joystick for TestJoystick {
        fn read(&self) -> (i32, i32) {
            (0, 0)
        }
    }
}