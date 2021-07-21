use crate::control::Action::CheckButtons;
use crate::event_loop::EDT;
use crate::led::Led;
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
            self.led.modify(&|current: u8| {
                if current < 32 { current + 1 } else { current }
            });
            self.rgb.set_rgb(GREEN);
            self.remove_blinks();
            self.edt.schedule(DELAY_BLINK, Action::Blink { color: GREEN, blinks: 5 });
        }

        if self.minus_pin.is_down() {
            self.led.modify(&|current: u8| {
                if current > 0 { current - 1 } else { current }
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

        self.edt.schedule(DELAY_CHECK_BUTTONS, CheckButtons);
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