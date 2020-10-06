use std::{
    thread::sleep,
    time::Duration,
};
use std::io::{stdout};

use crossterm::{
    ExecutableCommand,
    Result,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use crossterm::cursor::{MoveTo, position};
use crossterm::terminal::{Clear, ClearType};

use crate::led::{DummyLed, Led};
use crate::pin::{KeyboardPin, Pin};

mod pin;
mod led;

fn main() {
    event_loop()
}

fn event_loop() {
    let esc_pin = KeyboardPin::create(27);
    let minus_pin = KeyboardPin::create(37);
    let plus_pin = KeyboardPin::create(39);
    let led = DummyLed::create(0);

    stdout().execute(Print("Running light\n"));
    let (x, y) = position().unwrap();

    loop {
        sleep(Duration::from_millis(1_00));
        if esc_pin.is_down() { break; }

        if plus_pin.is_down() {
            led.modify(&|current: u8| {
                if current < 8 { current + 1 } else { current }
            });
        }

        if minus_pin.is_down() {
            led.modify(&|current: u8| {
                if current > 0 { current - 1 } else { current }
            });
        }

        render_flashlight_state(x, y, led.get_pwm());
    }
}

fn render_flashlight_state(x: u16, y: u16, pwm: u8) -> Result<()> {
    stdout()
        .execute(Clear(ClearType::FromCursorDown))?
        .execute(MoveTo(x, y))?
        .execute(SetForegroundColor(Color::Blue))?
        .execute(SetBackgroundColor(Color::Red))?
        .execute(Print(format!("pwm: {}", pwm)))?
        .execute(ResetColor);
    return Ok(());
}