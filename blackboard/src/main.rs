// make `std` available when testing
#![cfg_attr(not(test), no_std)]
#![no_main]

extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate jlink_rtt;
extern crate nb;
extern crate panic_halt;
extern crate stm32g0xx_hal as stm_hal;

use core::cell::{Cell, RefCell};
use core::fmt::Write;

use nb::block;
use rt::{entry, exception, ExceptionFrame};
use stm_hal::prelude::*;
use stm_hal::{hal, stm32};

use light_control::bsp::led::Led;
use light_control::control::LightControl;
use light_control::edt::{Event, EDT};

use crate::button::PullUpButton;
use crate::pwm_led::PwmLed;
use crate::rgb::GpioRgb;

mod button;
mod pwm_led;
mod rgb;

#[entry]
fn main() -> ! {
    let mut output = jlink_rtt::NonBlockingOutput::new();
    let _ = writeln!(output, "Firmware started!");

    // https://github.com/stm32-rs/stm32g0xx-hal
    let dp = stm32::Peripherals::take().expect("cannot take peripherals");
    let mut rcc = dp.RCC.constrain();

    let gpioa = dp.GPIOA.split(&mut rcc);
    let gpioc = dp.GPIOC.split(&mut rcc);
    let gpiob = dp.GPIOB.split(&mut rcc);

    let mut watchdog = dp.IWDG.constrain();
    watchdog.start(2000.ms());

    let mut timer = dp.TIM17.timer(&mut rcc);
    let edt = EDT::create();
    // 16 Khz is not very efficient, but also is not audible
    let pwm = dp.TIM1.pwm(16000.hz(), &mut rcc);
    let led_low = PwmLed::create(pwm.bind_pin(gpiob.pb3));
    let led_high = PwmLed::create(pwm.bind_pin(gpioa.pa8));
    let rgb = GpioRgb {
        pin: RefCell::new(gpioc.pc6.into_push_pull_output()),
        state: Cell::new(0),
    };

    let light_control = LightControl::new(
        PullUpButton {
            pin: gpiob.pb4.into_pull_up_input(),
        },
        PullUpButton {
            pin: gpioa.pa1.into_pull_up_input(),
        },
        PullUpButton {
            pin: gpioa.pa0.into_pull_up_input(),
        },
        &led_low,
        &led_high,
        &rgb,
        &edt,
    );

    light_control.start();
    light_control.jump_start();

    let mut prev_logged_state = (0, 0);
    loop {
        match edt.poll() {
            Event::Execute { msg } => {
                watchdog.feed();
                light_control.process_message(msg);
                if prev_logged_state != (led_high.get(), led_low.get()) {
                    writeln!(output, "high: {}%, low: {}%", led_high.get(), led_low.get()).unwrap();
                    prev_logged_state = (led_high.get(), led_low.get());
                }
            }
            Event::Wait { ms } => {
                timer.start(ms.ms());
                block!(timer.wait()).unwrap();
            }
            Event::Halt => {
                break;
            }
        }
    }

    writeln!(output, "Halted!").unwrap();
    panic!("Halted!");
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    let mut output = jlink_rtt::NonBlockingOutput::new();
    writeln!(output, "Hard fault {:#?}", ef).ok();
    panic!("Hard fault {:#?}", ef);
}
