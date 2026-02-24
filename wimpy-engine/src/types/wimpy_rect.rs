use std::ops::Mul;

use crate::WimpyVecAxis;

use super::WimpyVec;

#[derive(Debug,Copy,Clone,Default,PartialEq)]
pub struct WimpyRect {
    pub position: WimpyVec,
    pub size: WimpyVec,
}

impl WimpyRect  {

    pub const ZERO: Self = Self {
        position: WimpyVec::ZERO,
        size: WimpyVec::ZERO
    };

    pub const ONE: Self = Self {
        position: WimpyVec::ZERO,
        size: WimpyVec::ONE
    };

    pub fn x(&self) -> f32 {
        self.position.x
    }

    pub fn y(&self) -> f32 {
        self.position.y
    }

    pub fn width(&self) -> f32 {
        self.size.x
    }

    pub fn height(&self) -> f32 {
        self.size.y
    }

    pub fn top(&self) -> f32 {
        self.position.y
    }

    pub fn bottom(&self) -> f32 {
        self.position.y + self.size.y
    }

    pub fn left(&self) -> f32 {
        self.position.x
    }

    pub fn right(&self) -> f32 {
        self.position.x + self.size.x
    }

    pub fn center(&self) -> WimpyVec {
        self.size.mul_add(0.5,self.position)
    }

    pub fn center_x(&self) -> f32 {
        self.size.x.mul_add(0.5,self.position.x)
    }

    pub fn center_y(&self) -> f32 {
        self.size.y.mul_add(0.5,self.position.y)
    }

    pub fn clip(&self,vec: WimpyVec) -> WimpyVec {
        WimpyVec {
            x: vec.x.clamp(self.left(),self.right()),
            y: vec.y.clamp(self.top(),self.bottom()),
        }
    }

    pub fn origin_top_left_to_center(self) -> Self {
        Self {
            position: self.size.mul_add(0.5,self.position),
            size: self.size,
        }
    }

    pub fn origin_center_to_top_left(self) -> Self {
        Self {
            position: self.size.mul_add(-0.5,self.position),
            size: self.size,
        }
    }

    pub fn quadrant(&self,quadrant: WimpyRectQuadrant) -> Self {
        let size = self.size * 0.25;
        match quadrant {
            WimpyRectQuadrant::TopLeft => Self {
                position: self.position,
                size,
            },
            WimpyRectQuadrant::TopRight => Self {
                position: self.position + size.axis(WimpyVecAxis::X),
                size,
            },
            WimpyRectQuadrant::BottomLeft => Self {
                position: self.position + size.axis(WimpyVecAxis::Y),
                size,
            },
            WimpyRectQuadrant::BottomRight => Self {
                position: self.position + size,
                size,
            },
        }
    }
}

impl Mul<f32> for WimpyRect {
    type Output = Self;
    fn mul(self,rhs: f32) -> Self {
        Self {
            position: self.position * rhs,
            size: self.size * rhs,
        }
    }
}

impl Mul<WimpyVec> for WimpyRect {
    type Output = Self;
    fn mul(self,rhs: WimpyVec) -> Self {
        Self {
            position: self.position * rhs,
            size: self.size * rhs,
        }
    }
}

#[derive(Default,Clone,Copy)]
pub enum WimpyRectQuadrant {
    #[default]
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight
}
