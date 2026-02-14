#[derive(Clone,Copy)]
pub struct WimpyColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8
}

impl Default for WimpyColor {
    fn default() -> Self {
        Self {
            r: u8::MAX,
            g: u8::MAX,
            b: u8::MAX,
            a: u8::MAX
        }
    }
}

impl WimpyColor {

    pub fn get(r: u8,g: u8, b: u8,a: u8) -> WimpyColor {
        return WimpyColor { r, g, b, a };
    }

    pub fn decompose_float(self) -> [f32;4] {
        return [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ];
    }

    pub fn decompose(self) -> [u8;4] {
        return [
            self.r,
            self.g,
            self.b,
            self.a
        ]
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
        return Self {
            r: value.r as f64 / u8::MAX as f64,
            g: value.g as f64 / u8::MAX as f64,
            b: value.b as f64 / u8::MAX as f64,
            a: value.a as f64 / u8::MAX as f64
        }
    }
}
