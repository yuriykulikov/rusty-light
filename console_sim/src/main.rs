use std::cell::Cell;
use std::io;
use std::io::Stdout;
use std::thread::sleep;
use std::time::Duration;

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use keyboard_query::{DeviceQuery, DeviceState};
use light_control::battery_voltage_to_capacity::battery_voltage_to_capacity;
use tokio::time::Instant;
use tui::backend::CrosstermBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, Paragraph};
use tui::{Frame, Terminal};

use light_control::bsp::adc::Sensors;
use light_control::bsp::led::Led;
use light_control::bsp::rgb::{Rgb, BLUE, GREEN, RED};
use light_control::control::LightControl;
use light_control::edt::{Event, EDT};

use crate::dummy_led::DummyLed;
use crate::dummy_rgb::DummyRgb;
use crate::keyboard_pin::KeyboardPin;
use crate::power_dissipation::{battery_capacity, calculate_temperature};

mod dummy_led;
mod dummy_rgb;
mod keyboard_pin;
mod power_dissipation;

struct DummySensors {
    battery: Cell<u32>,
    temp: Cell<i32>,
}

impl Sensors for DummySensors {
    fn battery_voltage(&self, _high_percentage: u32, _low_percentage: u32) -> u32 {
        self.battery.get()
    }

    fn temp(&self) -> i32 {
        self.temp.get()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (minus_pin, plus_pin, toggle_pin) = keys();
    let led = DummyLed::create(0);
    let led_high = DummyLed::create(0);
    let rgb = DummyRgb::create();
    let edt = EDT::create();
    let sensors = DummySensors {
        battery: Cell::new(8000),
        temp: Cell::new(20),
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

    let kbd = DeviceState::new();
    let prev_drawn_state: Cell<(u32, u32, u8, i32, u32)> = Cell::new((0, 0, 0, 20, 8000));
    let mut since_last_temp_check = 0;
    loop {
        if kbd.get_keys().contains(&KEY_CODE_ESC) {
            break;
        }

        match kbd.get_keys().iter().next() {
            Some(&KEY_CODE_1) => sensors.battery.set(battery_capacity(2)),
            Some(&KEY_CODE_2) => sensors.battery.set(battery_capacity(4)),
            Some(&KEY_CODE_3) => sensors.battery.set(battery_capacity(7)),
            Some(&KEY_CODE_4) => sensors.battery.set(battery_capacity(10)),
            Some(&KEY_CODE_5) => sensors.battery.set(battery_capacity(15)),
            Some(&KEY_CODE_6) => sensors.battery.set(battery_capacity(20)),
            Some(&KEY_CODE_7) => sensors.battery.set(battery_capacity(30)),
            Some(&KEY_CODE_8) => sensors.battery.set(battery_capacity(40)),
            Some(&KEY_CODE_9) => sensors.battery.set(battery_capacity(80)),
            Some(&KEY_CODE_0) => sensors.battery.set(battery_capacity(100)),
            _ => {}
        }

        match edt.poll() {
            Event::Execute { msg } => light_control.process_message(msg),
            Event::Wait { ms } => {
                let ms = ms as u64;

                since_last_temp_check += ms;
                if since_last_temp_check > 250 {
                    let new_temp =
                        calculate_temperature(led.get(), led_high.get(), sensors.temp.get());
                    sensors.temp.set(new_temp);
                    since_last_temp_check = 0;
                }

                let start = Instant::now();
                let state_to_draw: (u32, u32, u8, i32, u32) = (
                    led.get(),
                    led_high.get(),
                    rgb.get_rgb(),
                    sensors.temp.get(),
                    sensors.battery.get(),
                );
                if prev_drawn_state.get() != state_to_draw {
                    draw_tui(
                        &mut terminal,
                        led.get(),
                        led_high.get(),
                        rgb.get_rgb(),
                        sensors.temp.get(),
                        sensors.battery.get(),
                    )?;
                }
                prev_drawn_state.set(state_to_draw);
                let duration_ms = start.elapsed().as_millis() as u64;
                if ms > duration_ms {
                    sleep(Duration::from_millis(ms - duration_ms));
                }
            }
            Event::Halt => {
                break;
            }
        }
    }

    disable_raw_mode().expect("can go back to normal");

    Ok(())
}

fn draw_tui(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    led: u32,
    led_high: u32,
    rgb: u8,
    temp: i32,
    bat: u32,
) -> io::Result<()> {
    terminal.draw(|rect| {
        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(4),
                    Constraint::Length(4),
                    Constraint::Length(4),
                ]
                .as_ref(),
            )
            .split(rect.size());

        let mut vertical_iter = vertical_layout.into_iter();
        render_led(led_high, rect, vertical_iter.next().unwrap());
        render_led(led, rect, vertical_iter.next().unwrap());
        render_rgb(rgb, rect, vertical_iter.next().unwrap());
        render_values_paragraph(
            led,
            led_high,
            temp,
            bat,
            rect,
            vertical_iter.next().unwrap(),
        );
        render_help(rect, vertical_iter.next().unwrap());
    })?;
    Ok(())
}

fn render_help(rect: &mut Frame<CrosstermBackend<Stdout>>, rect2: Rect) {
    let help_paragraph = Paragraph::new(vec![
        Spans::from(Span::raw(format!(
            "Buttons(click and long-click): ← (-) → (+) ↑ (toggle high beam) ↓ (toggle battery), ESC to terminate"
        ))),
    ])
        .alignment(Alignment::Left);
    rect.render_widget(help_paragraph, rect2);
}

fn render_values_paragraph(
    led: u32,
    led_high: u32,
    temp: i32,
    bat: u32,
    rect: &mut Frame<CrosstermBackend<Stdout>>,
    area: Rect,
) {
    let values_paragraph = Paragraph::new(vec![
        Spans::from(Span::raw(format!("High: {}", led_high))),
        Spans::from(Span::raw(format!("Low:  {}", led))),
        Spans::from(Span::raw(format!("Temp: {}", temp))),
        Spans::from(Span::raw(format!(
            "Bat:  {}",
            battery_voltage_to_capacity(bat)
        ))),
    ])
    .alignment(Alignment::Left);
    rect.render_widget(values_paragraph, area);
}

fn render_rgb(rgb: u8, rect: &mut Frame<CrosstermBackend<Stdout>>, rect1: Rect) {
    let rgb_style = Style::default().bg(Color::Rgb(
        if rgb & RED > 0 { 230 } else { 0 },
        if rgb & GREEN > 0 { 230 } else { 0 },
        if rgb & BLUE > 0 { 230 } else { 0 },
    ));
    let rgb_par = Paragraph::new(vec![Spans::from(Span::styled(
        format!("   LED   "),
        rgb_style,
    ))])
    .alignment(Alignment::Left);
    rect.render_widget(rgb_par, rect1);
}

/// Renders LED
///  ```
///   ╭───────────────────────────────────────────────────────────────────────────────────────╮
///   │                      ******************************************                       │
///   ╰───────────────────────────────────────────────────────────────────────────────────────╯
/// ```
fn render_led(led: u32, rect: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
    rect.render_widget(
        Paragraph::new(Spans::from(Span::styled(
            " ".repeat(led as usize),
            Style::default().bg(Color::Rgb(253, 244, 220)),
        )))
        .alignment(Alignment::Center)
        .block(
            // Block::default().borders(Borders::BOTTOM).border_type(BorderType::Plain)
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        ),
        area,
    );
}

#[cfg(target_os = "linux")]
const KEY_CODE_ESC: u16 = 1;
#[cfg(target_os = "linux")]
const KEY_CODE_RIGHT: u16 = 106;
#[cfg(target_os = "linux")]
const KEY_CODE_LEFT: u16 = 105;
// #[cfg(target_os = "linux")]
// const KEY_CODE_DOWN: u16 = 108;
#[cfg(target_os = "linux")]
const KEY_CODE_UP: u16 = 103;

#[cfg(target_os = "linux")]
const KEY_CODE_1: u16 = 2;
#[cfg(target_os = "linux")]
const KEY_CODE_2: u16 = 3;
#[cfg(target_os = "linux")]
const KEY_CODE_3: u16 = 4;
#[cfg(target_os = "linux")]
const KEY_CODE_4: u16 = 5;
#[cfg(target_os = "linux")]
const KEY_CODE_5: u16 = 6;
#[cfg(target_os = "linux")]
const KEY_CODE_6: u16 = 7;
#[cfg(target_os = "linux")]
const KEY_CODE_7: u16 = 8;
#[cfg(target_os = "linux")]
const KEY_CODE_8: u16 = 9;
#[cfg(target_os = "linux")]
const KEY_CODE_9: u16 = 10;
#[cfg(target_os = "linux")]
const KEY_CODE_0: u16 = 11;

#[cfg(target_os = "linux")]
fn keys() -> (KeyboardPin, KeyboardPin, KeyboardPin) {
    return (
        KeyboardPin::create(KEY_CODE_LEFT),
        KeyboardPin::create(KEY_CODE_RIGHT),
        KeyboardPin::create(KEY_CODE_UP),
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
