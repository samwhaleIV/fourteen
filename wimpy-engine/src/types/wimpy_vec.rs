use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Sub, SubAssign};
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

    pub fn from_axis(axis: WimpyVecAxis,value: f32) -> Self {
        match axis {
            WimpyVecAxis::X => Self {
                x: value,
                y: 0.0,
            },
            WimpyVecAxis::Y => Self {
                x: 0.0,
                y: value,
            },
        }
    }

    pub fn axis(self,axis: WimpyVecAxis) -> Self {
        match axis {
            WimpyVecAxis::X => Self {
                x: self.x,
                y: 0.0,
            },
            WimpyVecAxis::Y => Self {
                x: 0.0,
                y: self.y,
            },
        }
    }

    /// (self * a) + b
    pub fn mul_add(self,a: f32,b: Self) -> Self {
        Self {
            x: self.x.mul_add(a,b.x),
            y: self.y.mul_add(a,b.y)
        }
    }

    pub fn floor(self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor()
        }
    }

    pub fn ceil(self) -> Self {
        Self {
            x: self.x.ceil(),
            y: self.y.ceil()
        }
    }

    pub fn round(self) -> Self {
        Self {
            x: self.x.round(),
            y: self.y.round()
        }
    }

    pub fn clamp(self,min: f32,max: f32) -> Self {
        Self {
            x: self.x.clamp(min,max),
            y: self.y.clamp(min,max)
        }
    }

    pub fn reciprocal(self) -> Self {
        Self {
            x: 1.0 / self.x,
            y: 1.0 / self.y
        }
    }

    /// Gets the smaller of the two dimensions.
    pub fn smallest(self) -> f32 {
        self.x.min(self.y)
    }

    /// Gets the larger of the two dimensions.
    pub fn largest(self) -> f32 {
        self.x.max(self.y)
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
            y: self.y + rhs,
        }
    }
}

impl Add<Self> for WimpyVec {
    type Output = Self;
    fn add(self,rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
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

impl From<[i32;2]> for WimpyVec {
    fn from(value: [i32;2]) -> Self {
        Self {
            x: value[0] as f32,
            y: value[1] as f32
        }
    }
}

impl From<[u32;2]> for WimpyVec {
    fn from(value: [u32;2]) -> Self {
        Self {
            x: value[0] as f32,
            y: value[1] as f32
        }
    }
}

impl From<[u16;2]> for WimpyVec {
    fn from(value: [u16;2]) -> Self {
        Self {
            x: value[0] as f32,
            y: value[1] as f32
        }
    }
}

impl From<[u8;2]> for WimpyVec {
    fn from(value: [u8;2]) -> Self {
        Self {
            x: value[0] as f32,
            y: value[1] as f32
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

impl From<u32> for WimpyVec {
    fn from(value: u32) -> Self {
        Self {
            x: value as f32,
            y: value as f32,
        }
    }
}

impl From<f32> for WimpyVec {
    fn from(value: f32) -> Self {
        Self {
            x: value,
            y: value,
        }
    }
}

impl From<i32> for WimpyVec {
    fn from(value: i32) -> Self {
        Self {
            x: value as f32,
            y: value as f32,
        }
    }
}

#[derive(Default,Copy,Clone)]
pub enum WimpyVecAxis {
    #[default]
    X,
    Y
}

impl Index<WimpyVecAxis> for WimpyVec {
    type Output = f32;
    fn index(&self,index: WimpyVecAxis) -> &f32 {
        match index {
            WimpyVecAxis::X => &self.x,
            WimpyVecAxis::Y => &self.y,
        }
    }
}

impl IndexMut<WimpyVecAxis> for WimpyVec {
    fn index_mut(&mut self,index: WimpyVecAxis) -> &mut f32 {
        match index {
            WimpyVecAxis::X => &mut self.x,
            WimpyVecAxis::Y => &mut self.y,
        }
    }
}
