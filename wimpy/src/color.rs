#![allow(dead_code,unused_variables)]
use std::u8;

pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8
}

const FULL_OPACITY: u8 = u8::MAX;

impl Default for Color {
    fn default() -> Self {
        Self {
            r: u8::MAX,
            g: u8::MAX,
            b: u8::MAX,
            a: u8::MAX
        }
    }
}

impl Color {

    pub fn get(r: u8,g: u8, b: u8,a: u8) -> Color {
        return Color { r, g, b, a };
    }

    pub fn to_float_array(&self) -> [f32;4] {
        return [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ];
    }

    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: u8::MAX
    };

    pub const WHITE: Self = Self {
        r: u8::MAX,
        g: u8::MAX,
        b: u8::MAX,
        a: u8::MAX
    };

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
