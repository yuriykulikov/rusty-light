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
use rt::{entry, exception, ExceptionFrame};

use light_control::control::{Action, LightControl};
use light_control::control::Action::{Blink, SetPwm};
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
    let mut green_led = gpioc.pc6.into_push_pull_output();

    let pwm = dp.TIM1.pwm(10.khz(), &mut rcc);
    let mut pwm_ch1 = pwm.bind_pin(gpiob.pb3);

    let max = pwm_ch1.get_max_duty();

    let mut duties: [u16; 101] = [0; 101];
    for i in 0..=100 {
        let duh = i as u32;
        let m = max as u32;
        let duty = if i < 8 { m * duh / 903 } else { m * (duh + 16) / 116 * (duh + 16) / 116 * (duh + 16) / 116 };
        duties[i] = duty as u16;
    }

    pwm_ch1.set_duty(0);
    pwm_ch1.enable();

    // let mut watchdog = dp.WWDG.constrain(&mut rcc);
    let mut watchdog = dp.IWDG.constrain();
    watchdog.start(2000.ms());

    let mut timer = dp.TIM17.timer(&mut rcc);

    let mut down = false;
    loop {
        match edt.poll() {
            Event::Execute { msg } => {
                watchdog.feed();
                writeln!(output, "Action: {:?}", msg);
                match msg {
                    Action::Blink { color, blinks } => {
                        if blinks == 1 {
                            green_led.set_high().unwrap();
                            edt.schedule(500, Blink { color: 0, blinks: 0 });
                        } else {
                            green_led.set_low().unwrap();
                            edt.schedule(500, Blink { color: 0, blinks: 1 });
                        }
                    }
                    Action::SetPwm { power_level: goal } => {
                        let duty = duties[goal as usize];
                        // writeln!(output, "Duty: {}", duty);
                        pwm_ch1.set_duty(max - duty);
                        if goal == 100 {
                            down = true;
                            edt.schedule(10, SetPwm { power_level: 99 });
                        } else if goal == 0 {
                            down = false;
                            edt.schedule(10, SetPwm { power_level: 1 });
                        } else if down {
                            edt.schedule(10, SetPwm { power_level: goal - 1 });
                        } else {
                            edt.schedule(10, SetPwm { power_level: goal + 1 });
                        }
                    }
                    _ => {}
                }
            }
            Event::Wait { ms } => {
                writeln!(output, "Going to sleep: {}", ms);
                timer.start(ms.ms());
                block!(timer.wait()).unwrap();
            }
            Event::Halt => {
                break;
            }
        }
    }
    writeln!(output, "Halted!");
    panic!("Halted!");
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    let mut output = jlink_rtt::Output::new();
    writeln!(output, "Hard fault {:#?}", ef).ok();
    panic!("Hard fault {:#?}", ef);
}



