use std::cell::Cell;
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
use light_control::bsp::led::{Led, MAX};
use light_control::bsp::pin::Pin;
use light_control::bsp::rgb::{BLUE, GREEN, RED, Rgb};
use light_control::control::LightControl;
use light_control::edt::EDT;

use crate::dummy_led::DummyLed;
use crate::dummy_rgb::DummyRgb;
use crate::keyboard_pin::KeyboardPin;

mod keyboard_pin;
mod dummy_rgb;
mod dummy_led;

fn main() {
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
        led_level: Cell::new(0),
    };
    light_control.start();

    let (x, y) = position().unwrap();

    let last_pwm = Cell::new(100 as u32);
    let last_rgb = Cell::new(7 as u8);

    let handler = &|action| {
        light_control.process_message(action);
        let new_pwm = led.get();
        let new_rgb = rgb.get_rgb();
        let prev_pwm = last_pwm.replace(new_pwm);
        let prev_rgb = last_rgb.replace(new_rgb);

        if prev_pwm != new_pwm || prev_rgb != new_rgb {
            render_flashlight_state(x, y, new_pwm, new_rgb).unwrap();
        }
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

fn render_flashlight_state(x: u16, y: u16, power_output: u32, rgb: u8) -> Result<()> {
    let red_led_color = if rgb & RED > 0 { Color::Red } else { Color::Black };
    let red_green_color = if rgb & GREEN > 0 { Color::Green } else { Color::Black };
    let red_blue_color = if rgb & BLUE > 0 { Color::Blue } else { Color::Black };

    let mut led_str = String::new();
    for _ in 0..power_output { led_str.push('*'); }
    for _ in 0..(MAX - power_output) { led_str.push(' '); }

    stdout()
        .execute(Clear(ClearType::FromCursorDown))?
        .execute(MoveTo(x, y))?
        .execute(Print(format!("{:2}%", power_output)))?
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