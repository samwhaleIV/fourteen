#![allow(dead_code,unused_variables)]
use std::u8;

pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8
}

const FULL_OPACITY: u8 = u8::MAX;

impl Color {

    pub fn get(r: u8,g: u8, b: u8,a: u8) -> Color {
        return Color { r, g, b, a };
    }

    pub const RED: Self = Self {
        r: u8::MAX,
        g: 0,
        b: 0,
        a: FULL_OPACITY,
    };
    pub const GREEN: Self = Self {
        r: 0,
        g: u8::MAX,
        b: 0,
        a: FULL_OPACITY,
    };
    pub const BLUE: Self = Self {
        r: 0,
        g: 0,
        b: u8::MAX,
        a: FULL_OPACITY,
    };

    pub const YELLOW: Self = Self {
        r: u8::MAX,
        g: u8::MAX,
        b: 0,
        a: FULL_OPACITY,
    };
    pub const CYAN: Self = Self {
        r: 0,
        g: u8::MAX,
        b: u8::MAX,
        a: FULL_OPACITY,
    };
    
    pub const MAGENTA: Self = Self {
        r: u8::MAX,
        g: 0,
        b: u8::MAX,
        a: FULL_OPACITY,
    };
}
