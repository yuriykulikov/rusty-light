
pub mod pin {
    /// A pin (of a button) which may be down (tied to the ground) or up (floating pin)
    pub trait Pin {
        fn is_down(&self) -> bool;
    }
}

pub mod rgb {
    pub const RED: u8 = 0x4;
    pub const GREEN: u8 = 0x02;
    pub const BLUE: u8 = 0x01;

    /// Power LED of the flashlight. [pwn] represents the duty cycle.
    pub trait Rgb {
        fn set_rgb(&self, rgb: u8);
        fn get_rgb(&self) -> u8;
    }
}

pub mod led {
    pub const MAX: u32 = 100;

    /// Power LED of the flashlight. [pwn] represents the duty cycle.
    pub trait Led {
        fn set(&self, duty_cycle: u32);
        fn get(&self) -> u32;
    }
}
