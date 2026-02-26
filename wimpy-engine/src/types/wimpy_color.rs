use std::u8;
use fast_srgb8::srgb8_to_f32;

#[derive(Clone,Copy)]
pub struct WimpyColorSrgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8
}

#[derive(Clone,Copy)]
pub struct WimpyColorLinear {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32
}

#[derive(Copy,Clone,Debug,Default)]
pub enum WimpyOpacity {
    #[default]
    Opaque,
    Percent95,
    Percent90,
    Percent75,
    Percent50,
    Percent25,
    Percent10,
    Percent5,
    Transparent
}

const L: u8 = u8::MIN;  // Low
const M: u8 = 128;      // Medium
const H: u8 = u8::MAX;  // High
const Q: u8 = 64;       // Quarter

impl Default for WimpyColorSrgb {
    fn default() -> Self {
        Self::WHITE
    }
}

impl Default for WimpyColorLinear {
    fn default() -> Self {
        Self::WHITE
    }
}

const fn srgb_u8(r: u8,g: u8,b: u8) -> WimpyColorSrgb {
    WimpyColorSrgb {
        r,
        g,
        b,
        a: u8::MAX
    }
}

const U8_MAX_RECIP: f32 = 1.0 / u8::MAX as f32;

const fn srgb_u8_to_linear_f32(color: WimpyColorSrgb) -> WimpyColorLinear {
    WimpyColorLinear {
        r: srgb8_to_f32(color.r),
        g: srgb8_to_f32(color.g),
        b: srgb8_to_f32(color.b),
        a: color.a as f32 * U8_MAX_RECIP
    }
}

#[derive(Default,Copy,Clone)]
pub enum WimpyNamedColor {
    #[default]
    White,
    Black,
    Gray,
    Red,
    Orange,
    Yellow,
    Green,
    Cyan,
    Blue,
    Magenta,
    Violet,
    Pink,
    Salmon,
    Grape,
    Jade,
    Brown,
    Lavender,
    Banana,
    Maroon,
    Navy
}

impl WimpyColorSrgb {
    pub const WHITE: Self =     srgb_u8(H,H,H);
    pub const BLACK: Self =     srgb_u8(L,L,L);
    pub const GRAY: Self =      srgb_u8(M,M,M);
    pub const RED: Self =       srgb_u8(H,L,L);
    pub const GREEN: Self =     srgb_u8(L,H,L);
    pub const BLUE: Self =      srgb_u8(L,L,H);
    pub const ORANGE: Self =    srgb_u8(H,M,L);
    pub const VIOLET: Self =    srgb_u8(M,L,H);
    pub const YELLOW: Self =    srgb_u8(H,H,L);
    pub const CYAN: Self =      srgb_u8(L,H,H);
    pub const MAGENTA: Self =   srgb_u8(H,L,H);
    pub const PINK: Self =      srgb_u8(H,M,H);
    pub const SALMON: Self =    srgb_u8(H,M,M);
    pub const GRAPE: Self =     srgb_u8(M,L,M);
    pub const JADE: Self =      srgb_u8(L,M,L);
    pub const BROWN: Self =     srgb_u8(M,Q,L);
    pub const LAVENDER: Self =  srgb_u8(M,M,H);
    pub const BANANA: Self =    srgb_u8(H,H,L);
    pub const MAROON: Self =    srgb_u8(M,L,L);
    pub const NAVY: Self =      srgb_u8(L,L,M);
}

impl WimpyColorLinear {
    pub const WHITE: Self =     srgb_u8_to_linear_f32(WimpyColorSrgb::WHITE);
    pub const BLACK: Self =     srgb_u8_to_linear_f32(WimpyColorSrgb::BLACK);
    pub const GRAY: Self =      srgb_u8_to_linear_f32(WimpyColorSrgb::GRAY);
    pub const RED: Self =       srgb_u8_to_linear_f32(WimpyColorSrgb::RED);
    pub const GREEN: Self =     srgb_u8_to_linear_f32(WimpyColorSrgb::GREEN);
    pub const BLUE: Self =      srgb_u8_to_linear_f32(WimpyColorSrgb::BLUE);
    pub const ORANGE: Self =    srgb_u8_to_linear_f32(WimpyColorSrgb::ORANGE);
    pub const VIOLET: Self =    srgb_u8_to_linear_f32(WimpyColorSrgb::VIOLET);
    pub const YELLOW: Self =    srgb_u8_to_linear_f32(WimpyColorSrgb::YELLOW);
    pub const CYAN: Self =      srgb_u8_to_linear_f32(WimpyColorSrgb::CYAN);
    pub const MAGENTA: Self =   srgb_u8_to_linear_f32(WimpyColorSrgb::MAGENTA);
    pub const PINK: Self =      srgb_u8_to_linear_f32(WimpyColorSrgb::PINK);
    pub const SALMON: Self =    srgb_u8_to_linear_f32(WimpyColorSrgb::SALMON);
    pub const GRAPE: Self =     srgb_u8_to_linear_f32(WimpyColorSrgb::GRAPE);
    pub const JADE: Self =      srgb_u8_to_linear_f32(WimpyColorSrgb::JADE);
    pub const BROWN: Self =     srgb_u8_to_linear_f32(WimpyColorSrgb::BROWN);
    pub const LAVENDER: Self =  srgb_u8_to_linear_f32(WimpyColorSrgb::LAVENDER);
    pub const BANANA: Self =    srgb_u8_to_linear_f32(WimpyColorSrgb::BANANA);
    pub const MAROON: Self =    srgb_u8_to_linear_f32(WimpyColorSrgb::MAROON);
    pub const NAVY: Self =      srgb_u8_to_linear_f32(WimpyColorSrgb::NAVY);
}

impl From<WimpyNamedColor> for WimpyColorSrgb {
    fn from(value: WimpyNamedColor) -> Self {
        use WimpyNamedColor::*;
        match value {
            White =>    WimpyColorSrgb::WHITE,
            Black =>    WimpyColorSrgb::BLACK,
            Gray =>     WimpyColorSrgb::GRAY,
            Red =>      WimpyColorSrgb::RED,
            Orange =>   WimpyColorSrgb::ORANGE,
            Yellow =>   WimpyColorSrgb::YELLOW,
            Green =>    WimpyColorSrgb::GREEN,
            Cyan =>     WimpyColorSrgb::CYAN,
            Blue =>     WimpyColorSrgb::BLUE,
            Magenta =>  WimpyColorSrgb::MAGENTA,
            Violet =>   WimpyColorSrgb::VIOLET,
            Pink =>     WimpyColorSrgb::PINK,
            Salmon =>   WimpyColorSrgb::SALMON,
            Grape =>    WimpyColorSrgb::GRAPE,
            Jade =>     WimpyColorSrgb::JADE,
            Brown =>    WimpyColorSrgb::BROWN,
            Lavender => WimpyColorSrgb::LAVENDER,
            Banana =>   WimpyColorSrgb::BANANA,
            Maroon =>   WimpyColorSrgb::MAROON,
            Navy =>     WimpyColorSrgb::NAVY,
        }
    }
}

impl From<WimpyNamedColor> for WimpyColorLinear {
    fn from(value: WimpyNamedColor) -> Self {
        use WimpyNamedColor::*;
        match value {
            White =>    WimpyColorLinear::WHITE,
            Black =>    WimpyColorLinear::BLACK,
            Gray =>     WimpyColorLinear::GRAY,
            Red =>      WimpyColorLinear::RED,
            Orange =>   WimpyColorLinear::ORANGE,
            Yellow =>   WimpyColorLinear::YELLOW,
            Green =>    WimpyColorLinear::GREEN,
            Cyan =>     WimpyColorLinear::CYAN,
            Blue =>     WimpyColorLinear::BLUE,
            Magenta =>  WimpyColorLinear::MAGENTA,
            Violet =>   WimpyColorLinear::VIOLET,
            Pink =>     WimpyColorLinear::PINK,
            Salmon =>   WimpyColorLinear::SALMON,
            Grape =>    WimpyColorLinear::GRAPE,
            Jade =>     WimpyColorLinear::JADE,
            Brown =>    WimpyColorLinear::BROWN,
            Lavender => WimpyColorLinear::LAVENDER,
            Banana =>   WimpyColorLinear::BANANA,
            Maroon =>   WimpyColorLinear::MAROON,
            Navy =>     WimpyColorLinear::NAVY,
        }
    }
}

impl From<WimpyOpacity> for f32 {
    fn from(value: WimpyOpacity) -> Self {
        match value {
            WimpyOpacity::Opaque =>         1.0,
            WimpyOpacity::Percent95 =>      0.95,
            WimpyOpacity::Percent90 =>      0.9,
            WimpyOpacity::Percent75 =>      0.75,
            WimpyOpacity::Percent50 =>      0.50,
            WimpyOpacity::Percent25 =>      0.25,
            WimpyOpacity::Percent10 =>      0.10,
            WimpyOpacity::Percent5 =>       0.05,
            WimpyOpacity::Transparent =>    0.0,
        }
    }
}

impl From<WimpyColorLinear> for [f32;4] {
    fn from(value: WimpyColorLinear) -> Self {
        [
            value.r,
            value.g,
            value.b,
            value.a
        ]
    }
}

impl From<WimpyColorLinear> for wgpu::Color {
    fn from(value: WimpyColorLinear) -> Self {
        wgpu::Color {
            r: value.r as f64,
            g: value.g as f64,
            b: value.b as f64,
            a: value.a as f64,
        }
    }
}

pub trait WimpyColor {
    fn into_linear(self) -> WimpyColorLinear;
}

impl WimpyColor for (WimpyNamedColor,WimpyOpacity) {
    fn into_linear(self) -> WimpyColorLinear {
        let mut color: WimpyColorLinear = self.0.into();
        color.a = self.1.into();
        color
    }
}

impl WimpyColor for WimpyNamedColor {
    fn into_linear(self) -> WimpyColorLinear {
        self.into()
    }
}

impl WimpyColor for WimpyColorLinear {
    fn into_linear(self) -> WimpyColorLinear {
        self
    }
}

impl WimpyColor for WimpyColorSrgb {
    fn into_linear(self) -> WimpyColorLinear {
        WimpyColorLinear {
            r: srgb8_to_f32(self.r),
            g: srgb8_to_f32(self.g),
            b: srgb8_to_f32(self.b),
            a: self.a as f32 * U8_MAX_RECIP
        }
    }
}
