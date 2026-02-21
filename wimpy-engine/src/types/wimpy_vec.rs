use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use super::*;

#[derive(Debug,Copy,Clone,Default,PartialEq)]
pub struct WimpyVec {
    pub x: f32,
    pub y: f32
}

impl WimpyVec {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0
    };

    pub const ONE_HALF: Self = Self {
        x: 0.5,
        y: 0.5
    };

    pub const ONE: Self = Self {
        x: 1.0,
        y: 1.0
    };

    pub const NEGATIVE_ONE: Self = Self {
        x: -1.0,
        y: -1.0
    };

    pub const NEGATIVE_ONE_HALF: Self = Self {
        x: -0.5,
        y: -0.5
    };

    /// (self * a) + b
    pub fn mul_add(self, a: f32, b: Self) -> Self {
        Self {
            x: self.x.mul_add(a,b.x),
            y: self.y.mul_add(a,b.y)
        }
    }
}

impl Mul<f32> for WimpyVec {
    type Output = Self;
    fn mul(self,rhs: f32) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<Self> for WimpyVec {
    type Output = Self;
    fn mul(self,rhs: Self) -> Self {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl Sub<f32> for WimpyVec {
    type Output = Self;

    fn sub(self,rhs: f32) -> Self {
        Self {
            x: self.x - rhs,
            y: self.y - rhs,
        }
    }
}

impl Sub<Self> for WimpyVec {
    type Output = Self;
    fn sub(self,rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Add<f32> for WimpyVec {
    type Output = Self;
    fn add(self,rhs: f32) -> Self {
        Self {
            x: self.x + rhs,
            y: self.x + rhs,
        }
    }
}

impl Add<Self> for WimpyVec {
    type Output = Self;
    fn add(self,rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.x + rhs.y,
        }
    }
}

impl Div<f32> for WimpyVec {
    type Output = Self;
    fn div(self,rhs: f32) -> Self {
        Self {
            x: self.x / rhs,
            y: self.y / rhs
        }
    }
}

impl Div<Self> for WimpyVec {
    type Output = Self;
    fn div(self,rhs: Self) -> Self {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y
        }
    }
}

impl AddAssign<f32> for WimpyVec {
    fn add_assign(&mut self,rhs: f32) {
        self.x += rhs;
        self.y += rhs;
    }
}

impl AddAssign<Self> for WimpyVec {
    fn add_assign(&mut self,rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl SubAssign<f32> for WimpyVec {
    fn sub_assign(&mut self,rhs: f32) {
        self.x -= rhs;
        self.y -= rhs;
    }
}

impl SubAssign<Self> for WimpyVec {
    fn sub_assign(&mut self,rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl DivAssign<f32> for WimpyVec {
    fn div_assign(&mut self,rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl DivAssign<Self> for WimpyVec {
    fn div_assign(&mut self,rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}

impl MulAssign<f32> for WimpyVec {
    fn mul_assign(&mut self,rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl MulAssign<Self> for WimpyVec {
    fn mul_assign(&mut self,rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl From<[f32;2]> for WimpyVec {
    fn from(value: [f32;2]) -> Self {
        Self {
            x: value[0],
            y: value[1],
        }
    }
}

impl From<IWimpyPoint> for WimpyVec {
    fn from(value: IWimpyPoint) -> Self {
        Self {
            x: value.x as f32,
            y: value.y as f32
        }
    }
}

impl From<UWimpyPoint> for WimpyVec {
    fn from(value: UWimpyPoint) -> Self {
        Self {
            x: value.x as f32,
            y: value.y as f32
        }
    }
}

impl From<WimpyVec> for [f32;2] {
    fn from(value: WimpyVec) -> Self {
        [
            value.x,
            value.y
        ]
    }
}
