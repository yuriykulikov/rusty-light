use crate::event_loop::EDT;
use crate::event_loop::Msg;
use crate::led::Led;
use crate::pin::Pin;
use crate::rgb::{BLUE, GREEN, RED, Rgb};

/// Control logic evaluates button states and changes the light intensity
pub struct LightControl<'a, P: Pin> {
    pub plus_pin: P,
    pub minus_pin: P,
    pub led: &'a dyn Led,
    pub rgb: &'a dyn Rgb,
    pub edt: &'a EDT,
}

const MSG_BLINK: i32 = 1;
const MSG_CHECK_BUTTONS: i32 = 2;

const DELAY_CHECK_BUTTONS: u32 = 75;
const DELAY_BLINK: u32 = 250;

impl<'a, P: Pin> LightControl<'a, P> {
    pub fn process_message(&self, msg: Msg) {
        match msg.what {
            MSG_CHECK_BUTTONS => self.check_buttons(),
            MSG_BLINK => self.blink_led(msg),
            _ => {}
        }
    }

    fn blink_led(&self, msg: Msg) {
        if msg.arg1 > 0 {
            let rgb = self.rgb.get_rgb();
            self.rgb.set_rgb(rgb ^ (msg.arg0 as u8));
            self.edt.schedule_with_args(
                DELAY_BLINK,
                MSG_BLINK,
                msg.arg0,
                msg.arg1 - 1,
            );
        }
    }

    pub fn check_buttons(&self) {
        if self.plus_pin.is_down() {
            self.led.modify(&|current: u8| {
                if current < 32 { current + 1 } else { current }
            });
            self.rgb.set_rgb(GREEN);
            self.edt.remove_with_what(MSG_BLINK);
            self.edt.schedule_with_args(DELAY_BLINK, MSG_BLINK, GREEN as i32, 5);
        }

        if self.minus_pin.is_down() {
            self.led.modify(&|current: u8| {
                if current > 0 { current - 1 } else { current }
            });

            if self.led.get_pwm() == 0 {
                self.rgb.set_rgb(BLUE);
                self.edt.remove_with_what(MSG_BLINK);
                self.edt.schedule_with_args(1000, MSG_BLINK, BLUE as i32, 1);
            } else {
                self.rgb.set_rgb(RED);
                self.edt.remove_with_what(MSG_BLINK);
                self.edt.schedule_with_args(DELAY_BLINK, MSG_BLINK, RED as i32, 5);
            }
        }

        self.edt.schedule(DELAY_CHECK_BUTTONS, MSG_CHECK_BUTTONS);
    }
}