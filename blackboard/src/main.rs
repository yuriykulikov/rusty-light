#![no_std]
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
use OutputPin;
use rt::{entry, exception, ExceptionFrame};
use stm_hal::gpio::{Input, Output, PullUp, PushPull};
use stm_hal::gpio::gpiob::{PB5, PB9};
use stm_hal::gpio::gpioc::PC6;
use stm_hal::prelude::*;
use stm_hal::stm32;
use stm_hal::stm32::TIM1;
use stm_hal::timer::Channel2;
use stm_hal::timer::pwm::PwmPin;

use light_control::bsp::led::Led;
use light_control::bsp::pin::Pin;
use light_control::bsp::rgb::Rgb;
use light_control::control::LightControl;
use light_control::edt::{EDT, Event};

#[entry]
fn main() -> ! {
    let mut output = jlink_rtt::NonBlockingOutput::new();
    let _ = writeln!(output, "Hello {}", 42);

    // https://github.com/stm32-rs/stm32g0xx-hal
    let dp = stm32::Peripherals::take().expect("cannot take peripherals");
    let mut rcc = dp.RCC.constrain();

    let gpioc = dp.GPIOC.split(&mut rcc);
    let gpiob = dp.GPIOB.split(&mut rcc);

    let mut watchdog = dp.IWDG.constrain();
    watchdog.start(2000.ms());

    let mut timer = dp.TIM17.timer(&mut rcc);
    let edt = EDT::create();
    let mut led = PwmLed::create(dp.TIM1.pwm(10.khz(), &mut rcc).bind_pin(gpiob.pb3));
    let mut rgb = GpioRgb { pin: RefCell::new(gpioc.pc6.into_push_pull_output()), state: Cell::new(0) };

    let light_control = LightControl::new(
        PlusButton { pin: gpiob.pb5.into_pull_up_input() },
        MinusButton { pin: gpiob.pb9.into_pull_up_input() },
        &mut led,
        &mut rgb,
        &edt,
    );

    light_control.start();

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
    let mut output = jlink_rtt::Output::new();
    writeln!(output, "Hard fault {:#?}", ef).ok();
    panic!("Hard fault {:#?}", ef);
}

struct PwmLed {
    duties: [u16; 101],
    pwm_ch: RefCell<PwmPin<TIM1, Channel2>>,
    state: Cell<u32>,
}

impl PwmLed {
    fn create(pwm_ch: PwmPin<TIM1, Channel2>) -> Self {
        let max = pwm_ch.get_max_duty();

        let mut led = PwmLed {
            duties: [0; 101],
            pwm_ch: RefCell::new(pwm_ch),
            state: Cell::new(0),
        };

        for i in 0..=100 {
            let duh = i as u32;
            let m = max as u32;
            let duty = if i < 8 { m * duh / 903 } else { m * (duh + 16) / 116 * (duh + 16) / 116 * (duh + 16) / 116 };
            led.duties[i] = duty as u16;
        }

        led.pwm_ch.borrow_mut().set_duty(max);
        led.pwm_ch.borrow_mut().enable();

        return led;
    }
}

impl Led for PwmLed {
    fn set(&self, pwm: u32) {
        self.state.set(pwm);
        self.pwm_ch.borrow_mut().set_duty(self.duties[100 - pwm as usize]);
    }

    fn get(&self) -> u32 {
        return self.state.get();
    }
}

struct GpioRgb {
    pin: RefCell<PC6<Output<PushPull>>>,
    state: Cell<u8>,
}

impl Rgb for GpioRgb {
    fn set_rgb(&self, rgb: u8) {
        self.state.set(rgb);
        if rgb == 0 {
            self.pin.borrow_mut().set_low().unwrap();
        } else {
            self.pin.borrow_mut().set_high().unwrap();
        }
    }

    fn get_rgb(&self) -> u8 {
        return self.state.get();
    }
}

struct PlusButton {
    pin: PB5<Input<PullUp>>,
}

impl Pin for PlusButton {
    fn is_down(&self) -> bool {
        return self.pin.is_low().unwrap_or(false);
    }
}

struct MinusButton {
    pin: PB9<Input<PullUp>>,
}

impl Pin for MinusButton {
    fn is_down(&self) -> bool {
        return self.pin.is_low().unwrap_or(false);
    }
}