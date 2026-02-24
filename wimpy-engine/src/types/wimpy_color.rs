use std::u8;

#[derive(Clone,Copy)]
pub struct WimpyColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8
}

const L: u8 = u8::MIN; // Low
const M: u8 = 128; // Medium
const H: u8 = u8::MAX; // High
const Q: u8 = 64; // Quarter

impl Default for WimpyColor {
    fn default() -> Self {
        Self::WHITE
    }
}

const fn rgb(r: u8,g: u8,b: u8) -> WimpyColor {
    WimpyColor { r, g, b, a: u8::MAX }
}

impl WimpyColor {
    pub const WHITE: Self =     rgb(H,H,H);
    pub const BLACK: Self =     rgb(L,L,L);
    pub const GRAY: Self =      rgb(M,M,M);
    pub const RED: Self =       rgb(H,L,L);
    pub const GREEN: Self =     rgb(L,H,L);
    pub const BLUE: Self =      rgb(L,L,H);
    pub const ORANGE: Self =    rgb(H,M,L);
    pub const VIOLET: Self =    rgb(M,L,H);
    pub const YELLOW: Self =    rgb(H,H,L);
    pub const CYAN: Self =      rgb(L,H,H);
    pub const MAGENTA: Self =   rgb(H,L,H);
    pub const PINK: Self =      rgb(H,M,H);
    pub const SALMON: Self =    rgb(H,M,M);
    pub const GRAPE: Self =     rgb(M,L,M);
    pub const JADE: Self =      rgb(L,M,L);
    pub const BROWN: Self =     rgb(M,Q,L);
    pub const LAVENDER: Self =  rgb(M,M,H);
    pub const BANANA: Self =    rgb(H,H,L);
    pub const MAROON: Self =    rgb(M,L,L);
    pub const NAVY: Self =      rgb(L,L,M);
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

impl From<WimpyNamedColor> for WimpyColor {
    fn from(value: WimpyNamedColor) -> Self {
        use WimpyNamedColor::*;
        match value {
            White =>    WimpyColor::WHITE,
            Black =>    WimpyColor::BLACK,
            Gray =>     WimpyColor::GRAY,
            Red =>      WimpyColor::RED,
            Orange =>   WimpyColor::ORANGE,
            Yellow =>   WimpyColor::YELLOW,
            Green =>    WimpyColor::GREEN,
            Cyan =>     WimpyColor::CYAN,
            Blue =>     WimpyColor::BLUE,
            Magenta =>  WimpyColor::MAGENTA,
            Violet =>   WimpyColor::VIOLET,
            Pink =>     WimpyColor::PINK,
            Salmon =>   WimpyColor::SALMON,
            Grape =>    WimpyColor::GRAPE,
            Jade =>     WimpyColor::JADE,
            Brown =>    WimpyColor::BROWN,
            Lavender => WimpyColor::LAVENDER,
            Banana =>   WimpyColor::BANANA,
            Maroon =>   WimpyColor::MAROON,
            Navy =>     WimpyColor::NAVY,
        }
    }
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

impl From<(WimpyNamedColor,WimpyOpacity)> for WimpyColor {
    fn from(value: (WimpyNamedColor,WimpyOpacity)) -> Self {
        use WimpyOpacity::*;
        let mut color: Self = value.0.into();
        color.a = match value.1 {
            Opaque =>       255,
            Percent95 =>    242,
            Percent90 =>    229,
            Percent75 =>    191,
            Percent50 =>    128,
            Percent25 =>    64,
            Percent10 =>    26,
            Percent5 =>     13,
            Transparent =>  0,
        };
        color
    }
}

impl From<WimpyColor> for wgpu::Color {
    fn from(value: WimpyColor) -> Self {
        Self {
            r: value.r as f64 / u8::MAX as f64,
            g: value.g as f64 / u8::MAX as f64,
            b: value.b as f64 / u8::MAX as f64,
            a: value.a as f64 / u8::MAX as f64
        }
    }
}

impl From<WimpyColor> for [f32;4] {
    fn from(value: WimpyColor) -> Self {
        [
            value.r as f32 / u8::MAX as f32,
            value.g as f32 / u8::MAX as f32,
            value.b as f32 / u8::MAX as f32,
            value.a as f32 / u8::MAX as f32,
        ]
    }
}

impl From<WimpyColor> for [u8;4] {
    fn from(value: WimpyColor) -> Self {
        [
            value.r,
            value.g,
            value.b,
            value.a,
        ]
    }
}
