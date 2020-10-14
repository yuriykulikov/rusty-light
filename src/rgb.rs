use std::cell::Cell;

pub const RED: u8 = 0x4;
pub const GREEN: u8 = 0x02;
pub const BLUE: u8 = 0x01;

/// Power LED of the flashlight. [pwn] represents the duty cycle.
pub trait Rgb {
    fn set_rgb(&self, rgb: u8);
    fn get_rgb(&self) -> u8;
}

/// Led which resides in memory, for simulation or testing
pub struct DummyRgb {
    rgb: Cell<u8>
}

impl DummyRgb {
    /// Factory function to create a dummy LED
    pub fn create() -> Self {
        return DummyRgb { rgb: Cell::new(0) };
    }
}

impl Rgb for DummyRgb {
    fn set_rgb(&self, rgb: u8) {
        self.rgb.set(rgb);
    }
    fn get_rgb(&self) -> u8 {
        return self.rgb.get();
    }
}

