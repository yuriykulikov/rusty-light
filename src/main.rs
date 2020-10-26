use std::io::stdout;
use std::thread::sleep;
use std::time::Duration;

use crossterm::{
    ExecutableCommand,
    Result,
    style::{Color, Print, ResetColor, SetBackgroundColor},
};
use crossterm::cursor::{MoveTo, position};
use crossterm::terminal::{Clear, ClearType};

use crate::control::LightControl;
use crate::event_loop::EDT;
use crate::led::{DummyLed, Led, PWM_MAX};
use crate::pin::{KeyboardPin, Pin};
use crate::rgb::{BLUE, DummyRgb, GREEN, RED, Rgb};

mod pin;
mod led;
mod control;
mod event_loop;
mod rgb;

fn main() {
    event_loop()
}

fn event_loop() {
    let esc_pin = KeyboardPin::create(27);
    let minus_pin = KeyboardPin::create(37);
    let plus_pin = KeyboardPin::create(39);
    let led = DummyLed::create(0);
    let rgb = DummyRgb::create();

    let edt = EDT::create();

    let light_control = LightControl {
        plus_pin,
        minus_pin,
        led: &led,
        edt: &edt,
        rgb: &rgb,
    };
    light_control.start();

    let (x, y) = position().unwrap();

    let handler = &|action| {
        light_control.process_message(action);
        render_flashlight_state(x, y, led.get_pwm(), rgb.get_rgb()).unwrap();
        if esc_pin.is_down() {
            edt.exit();
        }
    };

    let mut to_sleep = 0;
    loop {
        to_sleep = edt.process_events(to_sleep, handler);
        if to_sleep == 0 { break; }
        sleep(Duration::from_millis(to_sleep as u64));
    }

    println!("Finished!");
}

fn render_flashlight_state(x: u16, y: u16, pwm: u32, rgb: u8) -> Result<()> {
    let red_led_color = if rgb & RED > 0 { Color::Red } else { Color::Black };
    let red_green_color = if rgb & GREEN > 0 { Color::Green } else { Color::Black };
    let red_blue_color = if rgb & BLUE > 0 { Color::Blue } else { Color::Black };

    let mut led_str = String::new();
    for _ in 0..pwm { led_str.push('*'); }
    for _ in 0..(PWM_MAX - pwm) { led_str.push(' '); }

    stdout()
        .execute(Clear(ClearType::FromCursorDown))?
        .execute(MoveTo(x, y))?
        .execute(Print(format!("pwm: {:2}", pwm)))?
        .execute(Print(format!("  [{}]  ", led_str)))?
        .execute(Print("["))?
        .execute(SetBackgroundColor(red_led_color))?
        .execute(Print(" "))?
        .execute(SetBackgroundColor(red_green_color))?
        .execute(Print(" "))?
        .execute(SetBackgroundColor(red_blue_color))?
        .execute(Print(" "))?
        .execute(ResetColor)?
        .execute(Print("]"))
        .map(|_| ())
}