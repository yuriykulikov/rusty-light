use no_std_compat::cell::Cell;

use crate::bsp::adc::Sensors;
use crate::bsp::led::Led;
use crate::bsp::pin::Pin;
use crate::bsp::rgb::{Rgb, BLUE, GREEN, RED};
use crate::control::ButtonState::{Clicked, LongClicked, Nothing};
use crate::edt::EDT;

#[derive(Clone, Debug, Eq, PartialEq, Copy)]
pub enum Action {
    Blink {
        color: u8,
        blinks: u8,
        period: u16,
    },
    CheckButtons,
    SetPwm {
        start: u8,
        end: u8,
        i: usize,
        high_beam: bool,
    },
    IndicateBattery,
}

pub const BUTTON_CHECK_PERIOD: u32 = 50;
pub const LONG_CLICK_THRESHOLD: u32 = 1000;
pub const DELAY_BLINK: u16 = 100;

pub const POWER_LEVELS_LOW: &'static [u8] = &[0, 25, 50, 75, 100];
pub const POWER_LEVELS_LOW_AUX: &'static [u8] = &[0, 25, 45, 60, 80];
pub const POWER_LEVELS_HIGH: &'static [u8] = &[0, 60, 80, 90, 100];

pub const POWER_LEVEL_INIT: usize = 3;

pub const MAX_POWER_LEVEL: usize = POWER_LEVELS_LOW.len() - 1;
pub const ANIM_DURATION: u32 = 500;
const ANIM_SIZE: usize = (60 * ANIM_DURATION / 1000) as usize;
const ANIM_STEP: u32 = ANIM_DURATION / ANIM_SIZE as u32;

enum ButtonState {
    Nothing,
    Clicked,
    LongClicked,
}

/// Button which remembers how long has it been held down
struct StatefulButton<P: Pin> {
    pin: P,
    hold_time: Cell<u32>,
}

impl<P: Pin> StatefulButton<P> {
    fn check_state(&self, elapsed_time: u32) -> ButtonState {
        let held = self.hold_time.get();
        let pin_down = self.pin.is_down();

        self.hold_time
            .set(if pin_down { held + elapsed_time } else { 0 });

        if pin_down && held == LONG_CLICK_THRESHOLD {
            LongClicked
        } else if !pin_down && held > 1 && held < LONG_CLICK_THRESHOLD {
            Clicked
        } else {
            Nothing
        }
    }
}

#[derive(Copy, Clone)]
struct State {
    power_level: usize,
    high_beam: bool,
}

/// Control logic evaluates button states and changes the light intensity
pub struct LightControl<'a, P: Pin, M: Pin, T: Pin> {
    plus_pin: StatefulButton<P>,
    minus_pin: StatefulButton<M>,
    toggle_pin: StatefulButton<T>,
    led: &'a dyn Led,
    led_high: &'a dyn Led,
    rgb: &'a dyn Rgb,
    sensors: &'a dyn Sensors,
    edt: &'a EDT<Action>,
    state: Cell<State>,
}

impl<'a, P: Pin, M: Pin, T: Pin> LightControl<'a, P, M, T> {
    pub fn new(
        plus_pin: P,
        minus_pin: M,
        toggle_pin: T,
        led: &'a dyn Led,
        led_high: &'a dyn Led,
        rgb: &'a dyn Rgb,
        edt: &'a EDT<Action>,
        sensors: &'a dyn Sensors,
    ) -> Self {
        return LightControl {
            plus_pin: StatefulButton {
                pin: plus_pin,
                hold_time: Cell::new(0),
            },
            minus_pin: StatefulButton {
                pin: minus_pin,
                hold_time: Cell::new(0),
            },
            toggle_pin: StatefulButton {
                pin: toggle_pin,
                hold_time: Cell::new(0),
            },
            led,
            led_high,
            rgb,
            edt,
            sensors,
            state: Cell::new(State {
                power_level: 0,
                high_beam: false,
            }),
        };
    }

    pub fn start(&self) {
        self.check_buttons();
        self.indicate_battery_tick();
    }

    pub fn jump_start(&self) {
        self.state.set(State {
            power_level: POWER_LEVEL_INIT,
            high_beam: false,
        });

        self.edt.schedule(
            ANIM_STEP,
            Action::SetPwm {
                start: 0,
                end: POWER_LEVELS_HIGH[POWER_LEVEL_INIT],
                i: 0,
                high_beam: true,
            },
        );

        self.edt.schedule(
            ANIM_DURATION * 2,
            Action::SetPwm {
                start: POWER_LEVELS_HIGH[POWER_LEVEL_INIT],
                end: 0,
                i: 0,
                high_beam: true,
            },
        );

        self.edt.schedule(
            ANIM_DURATION,
            Action::SetPwm {
                start: 0,
                end: POWER_LEVELS_LOW[POWER_LEVEL_INIT],
                i: 0,
                high_beam: false,
            },
        );
    }

    pub fn process_message(&self, action: Action) {
        match action {
            Action::CheckButtons => self.check_buttons(),
            Action::Blink {
                color,
                blinks,
                period,
            } => self.blink_led(color, blinks, period),
            Action::SetPwm {
                start,
                end,
                i,
                high_beam,
            } => {
                self.continue_led_animation(start, end, i, high_beam);
            }
            Action::IndicateBattery => self.indicate_battery_tick(),
        }
    }

    fn blink_led(&self, color: u8, blinks: u8, period: u16) {
        if blinks > 0 {
            let rgb = self.rgb.get_rgb();
            let rgb = rgb ^ color;
            let rgb = if rgb == 0 && self.state.get().high_beam {
                BLUE
            } else {
                rgb
            };
            self.rgb.set_rgb(rgb);
            let action = Action::Blink {
                color,
                blinks: blinks - 1,
                period,
            };
            self.edt.schedule(period as u32, action);
        }
    }

    fn check_buttons(&self) {
        match self.minus_pin.check_state(BUTTON_CHECK_PERIOD) {
            Clicked => self.on_minus_clicked(),
            LongClicked => self.on_long_clicked(),
            Nothing => {}
        }

        match self.plus_pin.check_state(BUTTON_CHECK_PERIOD) {
            Clicked => self.on_plus_clicked(),
            LongClicked => self.on_long_clicked(),
            Nothing => {}
        }

        match self.toggle_pin.check_state(BUTTON_CHECK_PERIOD) {
            Clicked => self.on_toggle_clicked(),
            LongClicked => {}
            Nothing => {}
        }

        self.edt.schedule(BUTTON_CHECK_PERIOD, Action::CheckButtons);
    }

    fn on_plus_clicked(&self) {
        if self.state.get().power_level < MAX_POWER_LEVEL {
            self.increment_power_level();
            self.indicate_click();
        } else {
            self.indicate_nop();
        }
    }

    fn on_minus_clicked(&self) {
        if self.state.get().power_level > 1 {
            self.decrement_power_level();
            self.indicate_click();
        } else {
            self.indicate_nop();
        }
    }

    fn on_long_clicked(&self) {
        self.indicate_nop();
    }

    fn on_toggle_clicked(&self) {
        let current = self.state.get();
        self.change_state(State {
            high_beam: !current.high_beam,
            ..current
        });
        let rgb = self.rgb.get_rgb();
        if current.high_beam {
            self.rgb.set_rgb(rgb & !BLUE);
        } else {
            self.rgb.set_rgb(rgb | BLUE);
        }
    }

    fn remove_blinks(&self) {
        self.edt.remove(|action| match action {
            Action::Blink {
                color: _,
                blinks: _,
                period: _,
            } => true,
            _ => false,
        });
    }

    fn blink(&self, color: u8, times: u8, period: u16) {
        let rgb = self.rgb.get_rgb() & !BLUE;
        let rgb = rgb | color;
        self.rgb.set_rgb(rgb);
        self.remove_blinks();
        self.edt.schedule(
            period as u32,
            Action::Blink {
                color,
                blinks: times,
                period,
            },
        );
    }

    fn indicate_nop(&self) {
        self.blink(self.battery_color(), 7, DELAY_BLINK / 2);
    }

    fn indicate_click(&self) {
        self.blink(self.battery_color(), 1, 500);
    }

    fn indicate_battery_tick(&self) {
        self.indicate_click();
        self.edt.schedule(10000, Action::IndicateBattery);
    }

    fn battery_color(&self) -> u8 {
        let battery_voltage = self.sensors.battery_voltage();
        let color = if battery_voltage < 6500 {
            RED
        } else if battery_voltage < 7200 {
            RED | GREEN
        } else {
            GREEN
        };
        color
    }

    fn increment_power_level(&self) {
        let current = self.state.get();
        if current.power_level < MAX_POWER_LEVEL {
            self.change_state(State {
                power_level: current.power_level + 1,
                ..current
            });
        }
    }

    fn decrement_power_level(&self) {
        let current = self.state.get();
        if current.power_level > 0 {
            self.change_state(State {
                power_level: current.power_level - 1,
                ..current
            });
        }
    }

    fn change_state(&self, new_state: State) {
        self.edt.remove(|msg| match msg {
            Action::SetPwm {
                start: _,
                end: _,
                i: _,
                high_beam: _,
            } => true,
            _ => false,
        });
        if new_state.high_beam {
            self.animate_low_beam(POWER_LEVELS_LOW_AUX[new_state.power_level]);
            self.animate_high_beam(POWER_LEVELS_HIGH[new_state.power_level]);
        } else {
            self.animate_low_beam(POWER_LEVELS_LOW[new_state.power_level]);
            self.animate_high_beam(0);
        };
        self.state.set(new_state);
    }

    fn animate_high_beam(&self, end: u8) {
        self.continue_led_animation(self.led_high.get() as u8, end, 0, true);
    }

    fn animate_low_beam(&self, end: u8) {
        self.continue_led_animation(self.led.get() as u8, end, 0, false);
    }

    /// Calculates the pwm level for the given i, sets it and schedules the next step
    fn continue_led_animation(&self, start: u8, end: u8, i: usize, high_beam: bool) {
        let led = if high_beam { self.led_high } else { self.led };
        let diff = end as i32 - start as i32;
        let next_value = start as i32 + (diff * (i as i32) / ANIM_SIZE as i32);
        debug_assert!(next_value >= 0);
        led.set(next_value as u32);

        if i < ANIM_SIZE {
            let action = Action::SetPwm {
                start,
                end,
                i: i + 1,
                high_beam,
            };
            self.edt.schedule(ANIM_STEP, action);
        }
    }
}
