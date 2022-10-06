// make `std` available when testing
#![cfg_attr(not(test), no_std)]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;
extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate jlink_rtt;
extern crate nb;
extern crate panic_halt;
extern crate stm32g0xx_hal as stm_hal;

use core::cell::{Cell, RefCell};
use core::fmt::Write;

use alloc_cortex_m::CortexMHeap;
use nb::block;
use rt::{entry, exception, ExceptionFrame};
use stm_hal::analog::adc::{Adc, OversamplingRatio, Precision, SampleTime};
use stm_hal::prelude::*;
use stm_hal::{hal, stm32};

use light_control::bsp::adc::Sensors;
use light_control::bsp::led::Led;
use light_control::bsp::pin::Pin;
use light_control::bsp::rgb::Rgb;
use light_control::control::LightControl;
use light_control::edt::{Event, EDT};

use crate::adc::AdcSensors;
use crate::button::PullUpButton;
use crate::pwm_led::PwmLed;
use crate::rgb::GpioRgb;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();
const HEAP_SIZE: usize = 512; // in bytes

mod adc;
mod button;
mod pwm_led;
mod rgb;

#[entry]
fn main() -> ! {
    unsafe { ALLOCATOR.init(cortex_m_rt::heap_start() as usize, HEAP_SIZE) }

    // https://github.com/stm32-rs/stm32g0xx-hal
    let dp = stm32::Peripherals::take().unwrap();
    let cp = stm32::CorePeripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();

    let gpioa = dp.GPIOA.split(&mut rcc);
    let gpiob = dp.GPIOB.split(&mut rcc);

    let a0 = gpioa.pa0;
    let a1 = gpioa.pa1;
    let a6 = gpioa.pa7;

    let d9 = gpioa.pa8;
    let d11 = gpiob.pb5;
    let d12 = gpiob.pb4;
    let d13 = gpiob.pb3;

    // datasheet says so:
    // let d3 = gpiob.pb0;
    // let d4 = gpiob.pb7;
    // let d5 = gpiob.pb6;
    // but actually is it:
    let d3 = gpiob.pb1;
    // let d4 = gpioa.pa10;
    let d5 = gpioa.pa9;

    let mut watchdog = dp.IWDG.constrain();
    watchdog.start(2000.ms());

    let mut timer = dp.TIM17.timer(&mut rcc);
    let edt = EDT::create();
    // 16 Khz is not very efficient, but also is not audible
    let pwm = dp.TIM1.pwm(16000.hz(), &mut rcc);
    let led_low = PwmLed::create(pwm.bind_pin(d13));
    let led_high = PwmLed::create(pwm.bind_pin(d9));

    let rgb = GpioRgb {
        r: RefCell::new(d3.into_push_pull_output()),
        g: RefCell::new(d5.into_push_pull_output()),
        b: RefCell::new(d11.into_push_pull_output()),
        state: Cell::new(0),
    };

    rgb.set_rgb(0);

    let mut adc: Adc = dp.ADC.constrain(&mut rcc);
    adc.set_sample_time(SampleTime::T_80);
    adc.set_precision(Precision::B_12);
    adc.set_oversampling_ratio(OversamplingRatio::X_16);
    adc.set_oversampling_shift(16);
    adc.oversampling_enable(true);
    cp.SYST.delay(&mut rcc).delay(20.us());
    adc.calibrate();

    let sensors = AdcSensors {
        adc: RefCell::new(adc),
        vin_pin: RefCell::new(a6.into_analog()),
        vin_temp: RefCell::new(a1.into_analog()),
        r_pull_up: 10000,
        r_pull_down: 4700 + 90,
    };

    let light_control = LightControl::new(
        PullUpButton {
            pin: d12.into_pull_up_input(),
        },
        SensorPin { sensors: &sensors },
        PullUpButton {
            pin: a0.into_pull_up_input(),
        },
        &led_low,
        &led_high,
        &rgb,
        &edt,
        &sensors,
    );

    light_control.start();
    light_control.jump_start();

    let mut output = jlink_rtt::NonBlockingOutput::new();
    let mut prev_logged_time = 0;
    loop {
        match edt.poll() {
            Event::Execute { msg } => {
                watchdog.feed();
                light_control.process_message(msg);
                if edt.now() > prev_logged_time + 2000 {
                    writeln!(
                        output,
                        "h: {}% l: {}% v: {} t: {}",
                        led_high.get(),
                        led_low.get(),
                        sensors.battery_voltage(),
                        sensors.temp(),
                    )
                    .unwrap();
                    prev_logged_time = edt.now();
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
    panic!("");
}

#[exception]
fn HardFault(_ef: &ExceptionFrame) -> ! {
    panic!("");
}

#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    loop {}
}

struct SensorPin<'a> {
    sensors: &'a dyn Sensors,
}

impl Pin for SensorPin<'_> {
    fn is_down(&self) -> bool {
        self.sensors.temp() < 10
    }
}
