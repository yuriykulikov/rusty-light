use std::{io, thread};
use std::cell::Cell;
use std::io::{stdout, Stdout};
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::Receiver;
use std::thread::sleep;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event as CEvent, KeyCode, KeyEvent};
use crossterm::event::Event::Key;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use tui::backend::CrosstermBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::Terminal;
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, BorderType, Paragraph};

use light_control::bsp::led::{Led, MAX};
use light_control::bsp::pin::Pin;
use light_control::bsp::rgb::{BLUE, GREEN, RED, Rgb};
use light_control::control::LightControl;
use light_control::edt::{EDT, Event};

use crate::dummy_led::DummyLed;
use crate::dummy_rgb::DummyRgb;
use crate::keyboard_pin::KeyboardPin;

mod dummy_rgb;
mod dummy_led;
mod keyboard_pin;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (esc_pin, minus_pin, plus_pin) = keys();
    let led = DummyLed::create(0);
    let rgb = DummyRgb::create();

    let edt = EDT::create();

    let light_control = LightControl::new(
        plus_pin,
        minus_pin,
        &led,
        &rgb,
        &edt,
    );
    light_control.start();

    enable_raw_mode().expect("can run in raw mode");
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    loop {
        if esc_pin.is_down() { edt.exit(); }

        match edt.poll() {
            Event::Execute { msg } => { light_control.process_message(msg) }
            Event::Wait { ms } => { sleep(Duration::from_millis(ms as u64)) }
            Event::Halt => { break; }
        }

        draw_tui(&mut terminal, led.get(), rgb.get_rgb());
    }

    disable_raw_mode().expect("can go back to normal");

    Ok(())
}

fn draw_tui(terminal: &mut Terminal<CrosstermBackend<Stdout>>, led: u32, rgb: u8) -> io::Result<()> {
    terminal.draw(|rect| {
        let size = rect.size();
        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([Constraint::Length(3), Constraint::Min(2), Constraint::Length(3)].as_ref())
            .split(size);

        let brightness_paragraph = Paragraph::new(
            Spans::from(Span::styled(
                " ".repeat(led as usize),
                Style::default().bg(Color::Rgb(253, 244, 220)),
            ))
        )
            .alignment(Alignment::Center)
            .block(
                // Block::default().borders(Borders::BOTTOM).border_type(BorderType::Plain)
                Block::default().borders(Borders::ALL).border_type(BorderType::Rounded)
            );

        let mut rgb_style = Style::default().bg(Color::Rgb(
            if rgb & RED > 0 { 230 } else { 0 },
            if rgb & GREEN > 0 { 230 } else { 0 },
            if rgb & BLUE > 0 { 230 } else { 0 },
        ));

        let sim_paragraph = Paragraph::new(
            vec!(
                Spans::from(Span::raw(format!("brightness: {}", led))),
                Spans::from(Span::styled(format!("   LED   "), rgb_style)),
            )
        )
            .alignment(Alignment::Left)
            ;

        rect.render_widget(brightness_paragraph, vertical_layout[0]);
        rect.render_widget(sim_paragraph, vertical_layout[1]);
    })?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn keys() -> (KeyboardPin, KeyboardPin, KeyboardPin) {
    return (
        KeyboardPin::create(1),
        KeyboardPin::create(105),
        KeyboardPin::create(106),
    );
}

#[cfg(target_os = "windows")]
fn keys() -> (KeyboardPin, KeyboardPin, KeyboardPin) {
    return (
        KeyboardPin::create(27),
        KeyboardPin::create(37),
        KeyboardPin::create(39),
    );
}