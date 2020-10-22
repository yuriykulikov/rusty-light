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
    CheckButtons,
}

const DELAY_CHECK_BUTTONS: u32 = 75;
const DELAY_BLINK: u32 = 250;

const PWM_STEPS: &'static [u32] = &[0, 1, 4, 9, 16, 25, 36, 49, 64, 81, PWM_MAX];

impl<'a, P: Pin> LightControl<'a, P> {
    pub fn process_message(&self, action: Action) {
        match action {
            Action::CheckButtons => self.check_buttons(),
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

    pub fn check_buttons(&self) {
        if self.plus_pin.is_down() {
            self.on_plus_clicked();
        }

        if self.minus_pin.is_down() {
            self.on_minus_clicked()
        }

        self.edt.schedule(DELAY_CHECK_BUTTONS, CheckButtons);
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

    use crate::control::{LightControl};
    use crate::event_loop::EDT;
    use crate::led::{DummyLed, Led, PWM_MAX};
    use crate::pin::Pin;
    use crate::rgb::{DummyRgb, Rgb};

    struct TestPin<'a> {
        is_down: &'a Cell<bool>,
    }

    impl<'a> Pin for TestPin<'a> {
        /// returns true is pin is tied to the ground
        fn is_down(&self) -> bool {
            return self.is_down.get();
        }
    }

    /// TODO extract setup function
    #[test]
    fn button_clicks_change_brightness<'t>() {
        let plus_pin = Cell::new(false);
        let minus_pin = Cell::new(false);
        let led = DummyLed::create(0);
        let rgb = DummyRgb::create();
        let edt = EDT::create();
        let light_control = LightControl {
            plus_pin: TestPin { is_down: &plus_pin },
            minus_pin: TestPin { is_down: &minus_pin },
            led: &led,
            edt: &edt,
            rgb: &rgb,
        };
        light_control.check_buttons();

        let on_next_event = &|action| {
            light_control.process_message(action);
            render_flashlight_state(led.get_pwm(), rgb.get_rgb());
        };

        plus_pin.set(true);
        edt.process_events(800, on_next_event);
        assert_eq!(led.get_pwm(), 100);

        plus_pin.set(false);
        minus_pin.set(true);
        edt.process_events(800, on_next_event);
        assert_eq!(led.get_pwm(), 0);
    }

    fn render_flashlight_state(pwm: u32, rgb: u8) {
        let mut led_str = String::new();
        for _ in 0..pwm { led_str.push('*'); }
        for _ in 0..(PWM_MAX - pwm) { led_str.push(' '); }
        println!("  [{}]  [{}]", led_str, rgb);
    }
}