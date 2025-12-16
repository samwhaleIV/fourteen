#![allow(dead_code,unused_variables)]

type Unit = f32;

pub struct Layout {
    pub x: LayoutDimension,
    pub y: LayoutDimension
}

pub struct Area {
    pub x: Unit,
    pub y: Unit,
    pub width: Unit,
    pub height: Unit
}

impl Default for Area {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0
        }
    }
}

impl Area {
    pub fn one() -> Self {
        return Self::default()
    }

    pub fn to_center_encoded(&self) -> Self {
        return Self {
            x: self.x + (self.width * 0.5),
            y: self.y + (self.height * 0.5),
            width: self.width,
            height: self.height
        }
    }

    pub fn to_top_left_encoded(&self) -> Self {
        return Self {
            x: self.x + (self.width * -0.5),
            y: self.y + (self.height * -0.5),
            width: self.width,
            height: self.height
        }
    }
}

#[derive(Copy,Clone)]
pub enum Alignment {
    LeftToRight,
    RightToLeft,
    Center,
    CenterLeftToRight,
    CenterRightToLeft,
    Absolute
}

#[derive(Copy,Clone)]
pub enum SizeMode {
    Absolute,
    Relative
}

#[derive(Copy,Clone)]
pub struct Position {
    pub value: Size,
    pub alignment: Alignment
}

#[derive(Copy,Clone)]
pub struct Size {
    pub value: Unit,
    pub mode: SizeMode
}

#[derive(Copy,Clone)]
pub struct LayoutDimension {
    pub position: Position,
    pub size: Size,
    pub size_offset: Size,
}

impl Layout {
    //Top Left Encoded
    pub fn to_area(&self,parent: &Area) -> Area {  
        let (x,width) = calculate_area_dimension(parent.x,parent.width,self.x);
        let (y,height) = calculate_area_dimension(parent.y,parent.height,self.y);
        return Area { x, y, width, height };
    }
}

fn calculate_area_dimension(parent_position: Unit,parent_size: Unit,child: LayoutDimension) -> (Unit,Unit) {
    let mut size = dimension(parent_size,child.size);
    let mut position = position(parent_position,parent_size,size,child.position);

    /* Applies after all other layout calculation. */
    let size_offset = dimension(parent_size,child.size_offset);

    /* Inset or outset position based on the size change */
    position += size_offset * -0.5;

    size += size_offset;

    return (position,size);
}

fn dimension(parent_value: Unit,child: Size) -> Unit {
    return match child.mode {
        SizeMode::Absolute => child.value,
        SizeMode::Relative => parent_value * child.value,
    }
}

fn position(parent_position: Unit,parent_size: Unit,child_size: Unit,child_position: Position) -> Unit {

    let position_offset = dimension(parent_size,child_position.value);

    return match child_position.alignment {
        Alignment::Center => {
            //Translate to center of parent
            (parent_position + parent_size * 0.5) +
            //Align child on axis line
            (child_size * -0.5) +
            //Apply offset in regular LTR
            position_offset
        },
        
        Alignment::CenterLeftToRight => {
            //Translate to center of parent
            (parent_position + parent_size * 0.5) +
            //Apply offset in regular LTR
            position_offset
        },

        Alignment::CenterRightToLeft => {
            //Center of parent
            (parent_position + parent_size * 0.5) +
            //Push right edge to axis line
            (child_size * -1.0) +
            //Apply offset inverted because of RTL
            (position_offset * -1.0)
        },

        Alignment::LeftToRight => {
            //Parent position, which we inherit
            parent_position +
            //Apply offset in regular LTR
            position_offset
        },

        Alignment::RightToLeft => {
            //Translate to center of parent
            (parent_position + parent_size) +
            //Push right edge to axis line
            (child_size * -1.0) +
            //Apply offset inverted because of RTL
            (position_offset * -1.0)
        },

        /* Position is absolute, but the value itself can be parent size relative. */
        Alignment::Absolute => {
            //Apply offset in regular LTR. No constraint to parent bound
            position_offset
        },
    }
}
