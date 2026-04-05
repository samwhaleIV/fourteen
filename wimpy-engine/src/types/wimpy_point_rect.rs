use serde::Deserialize;
use super::UWimpyPoint;

#[derive(Deserialize,Debug,Copy,Clone,PartialEq,Eq)]
pub struct WimpyPointRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32
}

impl WimpyPointRect {
    pub const fn area_from_size(size: UWimpyPoint) -> Self {
        Self {
            x: 0,
            y: 0,
            width: size.x,
            height: size.y,
        }
    }
}
