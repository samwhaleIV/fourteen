use std::ops::{Add,Sub,AddAssign,SubAssign};

pub type IWimpyPoint = WimpyPoint<i32>;
pub type UWimpyPoint = WimpyPoint<u32>;

impl Default for IWimpyPoint {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}

impl Default for UWimpyPoint {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}

#[derive(Debug,Copy,Clone,Eq,PartialEq)]
pub struct WimpyPoint<T> {
    pub x: T,
    pub y: T,
}

impl IWimpyPoint {
    pub const ZERO: Self = Self {
        x: 0,
        y: 0
    };

    pub const ONE: Self = Self {
        x: 1,
        y: 1
    };

    pub const NEGATIVE_ONE: Self = Self {
        x: -1,
        y: -1
    };

    /// Gets the smaller of the two dimensions.
    pub fn smallest(&self) -> i32 {
        self.x.min(self.y)
    }

    /// Gets the larger of the two dimensions.
    pub fn largest(&self) -> i32 {
        self.x.max(self.y)
    }
}

impl UWimpyPoint {
    pub const ZERO: Self = Self {
        x: 0,
        y: 0
    };

    pub const ONE: Self = Self {
        x: 1,
        y: 1
    };

    /// Gets the smaller of the two dimensions.
    pub fn smallest(&self) -> u32 {
        self.x.min(self.y)
    }

    /// Gets the larger of the two dimensions.
    pub fn largest(&self) -> u32 {
        self.x.max(self.y)
    }
}

impl Add for IWimpyPoint {
    type Output = Self;
    fn add(self,rhs: Self) -> Self {
        Self {
            x: self.x.saturating_add(rhs.x),
            y: self.y.saturating_add(rhs.y)
        }
    }
}

impl Add for UWimpyPoint {
    type Output = Self;
    fn add(self,rhs: Self) -> Self {
        Self {
            x: self.x.saturating_add(rhs.x),
            y: self.y.saturating_add(rhs.y)
        }
    }
}

impl Sub for IWimpyPoint {
    type Output = Self;
    fn sub(self,rhs: Self) -> Self {
        Self {
            x: self.x.saturating_sub(rhs.x),
            y: self.y.saturating_sub(rhs.y)
        }
    }
}

impl Sub for UWimpyPoint {
    type Output = Self;
    fn sub(self,rhs: Self) -> Self {
        Self {
            x: self.x.saturating_sub(rhs.x),
            y: self.y.saturating_sub(rhs.y)
        }
    }
}

impl AddAssign for IWimpyPoint {
    fn add_assign(&mut self,rhs: Self) {
        self.x = self.x.saturating_add(rhs.x);
        self.y = self.y.saturating_add(rhs.y);
    }
}

impl AddAssign for UWimpyPoint {
    fn add_assign(&mut self,rhs: Self) {
        self.x = self.x.saturating_add(rhs.x);
        self.y = self.y.saturating_add(rhs.y);
    }
}

impl SubAssign for IWimpyPoint {
    fn sub_assign(&mut self,rhs: Self) {
        self.x = self.x.saturating_sub(rhs.x);
        self.y = self.y.saturating_sub(rhs.y);
    }
}

impl SubAssign for UWimpyPoint {
    fn sub_assign(&mut self,rhs: Self) {
        self.x = self.x.saturating_sub(rhs.x);
        self.y = self.y.saturating_sub(rhs.y);
    }
}

impl From<i32> for IWimpyPoint {
    fn from(value: i32) -> Self {
        Self {
            x: value,
            y: value
        }
    }
}

impl From<i64> for IWimpyPoint {
    fn from(value: i64) -> Self {
        Self {
            x: value as i32,
            y: value as i32
        }
    }
}

impl From<u32> for UWimpyPoint {
    fn from(value: u32) -> Self {
        Self {
            x: value,
            y: value
        }
    }
}

impl From<u64> for UWimpyPoint {
    fn from(value: u64) -> Self {
        Self {
            x: value as u32,
            y: value as u32
        }
    }
}

impl From<usize> for UWimpyPoint {
    fn from(value: usize) -> Self {
        Self {
            x: value as u32,
            y: value as u32
        }
    }
}

impl From<[u32;2]> for UWimpyPoint {
    fn from(value: [u32;2]) -> Self {
        Self {
            x: value[0],
            y: value[1]
        }
    }
}

impl From<[i32;2]> for IWimpyPoint {
    fn from(value: [i32;2]) -> Self {
        Self {
            x: value[0],
            y: value[1]
        }
    }
}
