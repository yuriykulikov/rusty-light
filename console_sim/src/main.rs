use std::cell::Cell;
use std::io;
use std::io::Stdout;
use std::thread::sleep;
use std::time::Duration;

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use tui::backend::CrosstermBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, Paragraph};
use tui::Terminal;

use light_control::bsp::adc::Sensors;
use light_control::bsp::led::Led;
use light_control::bsp::pin::Pin;
use light_control::bsp::rgb::{Rgb, BLUE, GREEN, RED};
use light_control::control::LightControl;
use light_control::edt::{Event, EDT};

use crate::dummy_led::DummyLed;
use crate::dummy_rgb::DummyRgb;
use crate::keyboard_pin::KeyboardPin;

mod dummy_led;
mod dummy_rgb;
mod keyboard_pin;

struct DummySensors {
    battery: Cell<u32>,
}

impl Sensors for DummySensors {
    fn battery_voltage(&self) -> u32 {
        self.battery.get()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (esc_pin, minus_pin, plus_pin, toggle_pin, battery_toggle) = keys();
    let led = DummyLed::create(0);
    let led_high = DummyLed::create(0);
    let rgb = DummyRgb::create();
    let edt = EDT::create();
    let sensors = DummySensors {
        battery: Cell::new(8000),
    };
    let light_control = LightControl::new(
        plus_pin, minus_pin, toggle_pin, &led, &led_high, &rgb, &edt, &sensors,
    );
    light_control.start();
    light_control.jump_start();

    enable_raw_mode().expect("can run in raw mode");
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let prev_drawn_state: Cell<(u32, u32, u8)> = Cell::new((0, 0, 0));
    loop {
        if esc_pin.is_down() {
            edt.exit();
        }

        if battery_toggle.is_down() {
            sensors.battery.set(match sensors.battery.get() {
                8000 => 7000,
                7000 => 6000,
                _ => 8000,
            });
        }

        match edt.poll() {
            Event::Execute { msg } => light_control.process_message(msg),
            Event::Wait { ms } => sleep(Duration::from_millis(ms as u64)),
            Event::Halt => {
                break;
            }
        }
        let state_to_draw: (u32, u32, u8) = (led.get(), led_high.get(), rgb.get_rgb());
        if prev_drawn_state.get() != state_to_draw {
            draw_tui(&mut terminal, led.get(), led_high.get(), rgb.get_rgb())?;
        }
        prev_drawn_state.set(state_to_draw);
    }

    disable_raw_mode().expect("can go back to normal");

    Ok(())
}

fn draw_tui(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    led: u32,
    led_high: u32,
    rgb: u8,
) -> io::Result<()> {
    terminal.draw(|rect| {
        let size = rect.size();
        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(2),
                    Constraint::Length(3),
                ]
                    .as_ref(),
            )
            .split(size);

        let brightness_paragraph = Paragraph::new(Spans::from(Span::styled(
            " ".repeat(led as usize),
            Style::default().bg(Color::Rgb(253, 244, 220)),
        )))
            .alignment(Alignment::Center)
            .block(
                // Block::default().borders(Borders::BOTTOM).border_type(BorderType::Plain)
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            );

        let brightness_high_paragraph = Paragraph::new(Spans::from(Span::styled(
            " ".repeat(led_high as usize),
            Style::default().bg(Color::Rgb(253, 244, 220)),
        )))
            .alignment(Alignment::Center)
            .block(
                // Block::default().borders(Borders::BOTTOM).border_type(BorderType::Plain)
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            );

        let rgb_style = Style::default().bg(Color::Rgb(
            if rgb & RED > 0 { 230 } else { 0 },
            if rgb & GREEN > 0 { 230 } else { 0 },
            if rgb & BLUE > 0 { 230 } else { 0 },
        ));

        let sim_paragraph = Paragraph::new(vec![
            Spans::from(Span::raw(format!("High: {}", led_high))),
            Spans::from(Span::raw(format!("Low: {}", led))),
            Spans::from(Span::styled(format!("   LED   "), rgb_style)),
            Spans::from(Span::raw(format!(
                "Buttons(click and long-click): ← (-) → (+) ↑ (toggle high beam) ↓ (toggle battery), ESC to terminate"
            ))),
        ])
            .alignment(Alignment::Left);

        rect.render_widget(brightness_high_paragraph, vertical_layout[0]);
        rect.render_widget(brightness_paragraph, vertical_layout[1]);
        rect.render_widget(sim_paragraph, vertical_layout[2]);
    })?;
    Ok(())
}

#[cfg(target_os = "linux")]
const KEY_CODE_ESC: u16 = 1;
#[cfg(target_os = "linux")]
const KEY_CODE_RIGHT: u16 = 106;
#[cfg(target_os = "linux")]
const KEY_CODE_LEFT: u16 = 105;
#[cfg(target_os = "linux")]
const KEY_CODE_DOWN: u16 = 108;
#[cfg(target_os = "linux")]
const KEY_CODE_UP: u16 = 103;

#[cfg(target_os = "linux")]
fn keys() -> (
    KeyboardPin,
    KeyboardPin,
    KeyboardPin,
    KeyboardPin,
    KeyboardPin,
) {
    return (
        KeyboardPin::create(KEY_CODE_ESC),
        KeyboardPin::create(KEY_CODE_LEFT),
        KeyboardPin::create(KEY_CODE_RIGHT),
        KeyboardPin::create(KEY_CODE_UP),
        KeyboardPin::create(KEY_CODE_DOWN),
    );
}

#[cfg(target_os = "windows")]
fn keys() -> (KeyboardPin, KeyboardPin, KeyboardPin) {
    return (
        KeyboardPin::create(27),
        KeyboardPin::create(37),
        KeyboardPin::create(39),
        KeyboardPin::create(38),
    );
}
