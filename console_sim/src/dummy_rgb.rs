use light_control::bsp::rgb::Rgb;
use std::cell::Cell;

/// Led which resides in memory, for simulation or testing
pub struct DummyRgb {
    rgb: Cell<u8>,
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
