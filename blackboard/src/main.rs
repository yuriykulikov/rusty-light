#![no_std]
#![no_main]

extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate jlink_rtt;
extern crate nb;
extern crate panic_halt;
extern crate stm32g0xx_hal as hal;

use core::fmt::Write;

use hal::prelude::*;
use hal::stm32;
use nb::block;
use rt::{entry, ExceptionFrame, exception};

#[entry]
fn main() -> ! {
    let mut output = jlink_rtt::NonBlockingOutput::new();
    let _ = writeln!(output, "Hello {}", 42);

    // https://github.com/stm32-rs/stm32g0xx-hal
    let dp = stm32::Peripherals::take().expect("cannot take peripherals");
    let mut rcc = dp.RCC.constrain();

    let gpioc = dp.GPIOC.split(&mut rcc);
    let gpiob = dp.GPIOB.split(&mut rcc);
    let mut green_led = gpioc.pc6.into_push_pull_output();

    let pwm = dp.TIM1.pwm(10.khz(), &mut rcc);
    let mut pwm_ch1 = pwm.bind_pin(gpiob.pb3);

    let max = pwm_ch1.get_max_duty();
    pwm_ch1.set_duty(max / 2);
    pwm_ch1.enable();

    // let mut watchdog = dp.WWDG.constrain(&mut rcc);
    let mut watchdog = dp.IWDG.constrain();
    watchdog.start(1000.ms());

    let mut timer = dp.TIM17.timer(&mut rcc);
    timer.start(250.ms());

    loop {
        watchdog.feed();
        let _ = writeln!(output, "Blink!");
        green_led.set_high().unwrap();
        for i in 1..4 {
            block!(timer.wait()).unwrap();
            pwm_ch1.set_duty(max / i);
        }
        green_led.set_low().unwrap();
        for i in (1..4).rev() {
            block!(timer.wait()).unwrap();
            pwm_ch1.set_duty(max / i);
        }
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    let mut output = jlink_rtt::Output::new();
    writeln!(output, "Hard fault {:#?}", ef).ok();
    panic!("Hard fault {:#?}", ef);
}



