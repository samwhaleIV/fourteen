use serde::Deserialize;

#[derive(Deserialize,Debug,Copy,Clone,PartialEq,Eq)]
pub struct WimpyPointRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32
}
