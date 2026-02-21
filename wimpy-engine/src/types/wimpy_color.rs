#[derive(Clone,Copy)]
pub struct WimpyColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8
}

impl Default for WimpyColor {
    fn default() -> Self {
        Self::WHITE
    }
}

impl WimpyColor {

    pub fn new(r: u8,g: u8,b: u8,a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: u8::MAX,
    };

    pub const WHITE: Self = Self {
        r: u8::MAX,
        g: u8::MAX,
        b: u8::MAX,
        a: u8::MAX,
    };

    pub const RED: Self = Self {
        r: u8::MAX,
        g: 0,
        b: 0,
        a: u8::MAX,
    };
    pub const GREEN: Self = Self {
        r: 0,
        g: u8::MAX,
        b: 0,
        a: u8::MAX,
    };
    pub const BLUE: Self = Self {
        r: 0,
        g: 0,
        b: u8::MAX,
        a: u8::MAX,
    };

    pub const YELLOW: Self = Self {
        r: u8::MAX,
        g: u8::MAX,
        b: 0,
        a: u8::MAX,
    };
    pub const CYAN: Self = Self {
        r: 0,
        g: u8::MAX,
        b: u8::MAX,
        a: u8::MAX,
    };
    
    pub const MAGENTA: Self = Self {
        r: u8::MAX,
        g: 0,
        b: u8::MAX,
        a: u8::MAX,
    };
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
            value.r as f32 / 255.0,
            value.g as f32 / 255.0,
            value.b as f32 / 255.0,
            value.a as f32 / 255.0,
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
