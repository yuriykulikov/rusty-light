use std::{
    thread::sleep,
    time::Duration,
};
use std::io::stdout;

use crossterm::{
    ExecutableCommand,
    Result,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use crossterm::cursor::{MoveTo, position};
use crossterm::terminal::{Clear, ClearType};

use crate::control::LightControl;
use crate::led::{DummyLed, Led};
use crate::pin::{KeyboardPin, Pin};

mod pin;
mod led;
mod control;

fn main() {
    event_loop()
}

fn event_loop() {
    let esc_pin = KeyboardPin::create(27);
    let minus_pin = KeyboardPin::create(37);
    let plus_pin = KeyboardPin::create(39);
    let led = DummyLed::create(0);

    let light_control = LightControl { plus_pin, minus_pin, led: &led };

    let (x, y) = position().unwrap();

    loop {
        sleep(Duration::from_millis(1_00));
        if esc_pin.is_down() { break; }
        light_control.tick();
        if render_flashlight_state(x, y, led.get_pwm()).is_err() { break; }
    }
}

fn render_flashlight_state(x: u16, y: u16, pwm: u8) -> Result<()> {
    stdout()
        .execute(Clear(ClearType::FromCursorDown))?
        .execute(MoveTo(x, y))?
        .execute(SetForegroundColor(Color::Blue))?
        .execute(SetBackgroundColor(Color::Red))?
        .execute(Print(format!("pwm: {}", pwm)))?
        .execute(ResetColor)
        .map(|_| ())
}