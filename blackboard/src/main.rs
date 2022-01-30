// make `std` available when testing
#![cfg_attr(not(test), no_std)]
#![no_main]

mod button;
mod joystick;
mod pwm_led;
mod rgb;

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
use stm_hal::analog::adc::{Adc, OversamplingRatio, Precision, SampleTime};
use stm_hal::prelude::*;
use stm_hal::{hal, stm32};

use crate::button::PullUpButton;
use crate::joystick::AdcJoystick;
use crate::pwm_led::PwmLed;
use crate::rgb::GpioRgb;
use light_control::control::LightControl;
use light_control::edt::{Event, EDT};

#[entry]
fn main() -> ! {
    let mut output = jlink_rtt::NonBlockingOutput::new();
    let _ = writeln!(output, "Firmware started!");

    // https://github.com/stm32-rs/stm32g0xx-hal
    let dp = stm32::Peripherals::take().expect("cannot take peripherals");
    let cp = stm32::CorePeripherals::take().expect("cannot take core peripherals");
    let mut rcc = dp.RCC.constrain();

    let gpioa = dp.GPIOA.split(&mut rcc);
    let gpioc = dp.GPIOC.split(&mut rcc);
    let gpiob = dp.GPIOB.split(&mut rcc);

    let mut watchdog = dp.IWDG.constrain();
    watchdog.start(2000.ms());

    let mut timer = dp.TIM17.timer(&mut rcc);
    let edt = EDT::create();
    let pwm = dp.TIM1.pwm(16000.hz(), &mut rcc);
    let mut led = PwmLed::create(pwm.bind_pin(gpiob.pb3));
    let mut led_high = PwmLed::create(pwm.bind_pin(gpioa.pa8));
    let mut rgb = GpioRgb {
        pin: RefCell::new(gpioc.pc6.into_push_pull_output()),
        state: Cell::new(0),
    };

    let mut adc: Adc = dp.ADC.constrain(&mut rcc);
    adc.set_sample_time(SampleTime::T_80);
    adc.set_precision(Precision::B_12);
    adc.set_oversampling_ratio(OversamplingRatio::X_16);
    adc.set_oversampling_shift(16);
    adc.oversampling_enable(true);
    cp.SYST.delay(&mut rcc).delay(20.us());
    adc.calibrate();

    let light_control = LightControl::new(
        PullUpButton {
            pin: gpiob.pb5.into_pull_up_input(),
        },
        PullUpButton {
            pin: gpiob.pb9.into_pull_up_input(),
        },
        PullUpButton {
            pin: gpiob.pb4.into_pull_up_input(),
        },
        AdcJoystick::create(gpioa.pa0.into_analog(), gpioa.pa1.into_analog(), adc),
        &mut led,
        &mut led_high,
        &mut rgb,
        &edt,
    );

    light_control.start();
    light_control.jump_start();

    loop {
        match edt.poll() {
            Event::Execute { msg } => {
                watchdog.feed();
                light_control.process_message(msg);
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
