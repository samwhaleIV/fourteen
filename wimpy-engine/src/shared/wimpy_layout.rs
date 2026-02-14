type Unit = f32;

pub struct WimpyLayout {
    pub x: LayoutDimension,
    pub y: LayoutDimension
}

#[derive(Copy,Clone)]
pub struct WimpyArea {
    pub x: Unit,
    pub y: Unit,
    pub width: Unit,
    pub height: Unit
}

impl Default for WimpyArea {
    fn default() -> Self {
        Self::ONE
    }
}

impl WimpyArea {
    pub const ONE: Self = Self {
        x: 0.0,
        y: 0.0,
        width: 1.0,
        height: 1.0
    };

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

    pub fn from_size(size: (Unit,Unit)) -> Self {
        return Self {
            x: 0.0,
            y: 0.0,
            width: size.0,
            height: size.1,
        }
    }

    pub fn multiply(self,value: Unit) -> Self {
        return Self {
            x: self.x * value,
            y: self.y * value,
            width: self.width * value,
            height: self.height * value,
        }
    }

    pub fn multiply_2d(self,value: (Unit,Unit)) -> Self {
        return Self {
            x: self.x * value.0,
            y: self.y * value.1,
            width: self.width * value.0,
            height: self.height * value.1,
        }
    }

    pub fn size(&self) -> (Unit,Unit) {
        return (self.width,self.height);
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
    RelativeWidth,
    RelativeHeight,
    RelativeSmallest,
    RelativeLargest
}

#[derive(Copy,Clone)]
pub struct Position {
    pub value: Size,
    pub alignment: Alignment
}

impl Default for Position {
    fn default() -> Self {
        Self {
            value: Default::default(),
            alignment: Alignment::Absolute
        }
    }
}

impl Position {
    pub fn center_of_parent() -> Self {
        return Self {
            value: Size::default(),
            alignment: Alignment::Center
        }
    }
    pub fn center_of_parent_with_offset(size: Size) -> Self {
        return Self {
            value: size,
            alignment: Alignment::Center
        }
    }
}

#[derive(Copy,Clone)]
pub struct Size {
    pub value: Unit,
    pub mode: SizeMode
}

impl Default for Size {
    fn default() -> Self {
        Self {
            value: 0.0,
            mode: SizeMode::Absolute
        }
    }
}

impl From<u32> for Size {
    fn from(value: u32) -> Self {
        return Self {
            value: value as Unit,
            mode: SizeMode::Absolute,
        }
    }
}

impl Size {
    pub fn of_parent_height(value: Unit) -> Self {
        return Self {
            value,
            mode: SizeMode::RelativeHeight,
        }
    }
    pub fn of_parent_width(value: Unit) -> Self {
        return Self {
            value,
            mode: SizeMode::RelativeWidth,
        }
    }
    pub fn of_parent_smallest(value: Unit) -> Self {
        return Self {
            value,
            mode: SizeMode::RelativeSmallest,
        }
    }
    pub fn of_parent_largest(value: Unit) -> Self {
        return Self {
            value,
            mode: SizeMode::RelativeLargest,
        }
    }
}

#[derive(Copy,Clone)]
pub struct LayoutDimension {
    pub position: Position,
    pub size: Size,
    pub size_offset: Size,
}

impl Default for LayoutDimension {
    fn default() -> Self {
        Self {
            position: Default::default(),
            size: Default::default(),
            size_offset: Default::default()
        }
    }
}

impl WimpyLayout {
    //Top Left Encoded
    pub fn compute(&self,parent: WimpyArea) -> WimpyArea {  
        let (x,width) = calculate_area_dimension(
            parent.x,
            parent.width,
            parent.size(),
            self.x
        );
        let (y,height) = calculate_area_dimension(
            parent.y,
            parent.height,
            parent.size(),
            self.y
        );
        return WimpyArea { x, y, width, height };
    }
    pub fn same_xy(layout_dimension: LayoutDimension) -> Self {
        return Self {
            x: layout_dimension,
            y: layout_dimension,
        }
    }
}

impl Default for WimpyLayout {
    fn default() -> Self {
        Self {
            x: LayoutDimension::default(),
            y: LayoutDimension::default(),
        }
    }
}

fn calculate_area_dimension(
    parent_position: Unit,
    parent_dimension: Unit,
    parent_size: (Unit,Unit),
    child: LayoutDimension
) -> (Unit,Unit) {
    let mut size = dimension(parent_size,child.size);
    let mut position = position(parent_position,parent_dimension,parent_size,size,child.position);

    /* Applies after all other layout calculation. */
    let size_offset = dimension(parent_size,child.size_offset);

    /* Inset or outset position based on the size change */
    position += size_offset * -0.5;

    size += size_offset;

    return (position,size);
}

fn dimension(
    parent_value: (Unit,Unit),
    child: Size
) -> Unit {
    return match child.mode {
        SizeMode::Absolute => {
            child.value
        },
        SizeMode::RelativeWidth => {
            parent_value.0 * child.value
        },
        SizeMode::RelativeHeight => {
            parent_value.1 * child.value
        },
        SizeMode::RelativeSmallest => {
            parent_value.0.min(parent_value.1) * child.value
        },
        SizeMode::RelativeLargest => {
            parent_value.0.max(parent_value.1) * child.value
        },
    }
}

fn position(
    parent_position: Unit,
    parent_dimension: Unit,
    parent_size: (Unit,Unit),
    child_size: Unit,
    child_position: Position
) -> Unit {

    let position_offset = dimension(parent_size,child_position.value);

    return match child_position.alignment {
        Alignment::Center => {
            //Translate to center of parent
            (parent_position + parent_dimension * 0.5) +
            //Align child on axis line
            (child_size * -0.5) +
            //Apply offset in regular LTR
            position_offset
        },
        
        Alignment::CenterLeftToRight => {
            //Translate to center of parent
            (parent_position + parent_dimension * 0.5) +
            //Apply offset in regular LTR
            position_offset
        },

        Alignment::CenterRightToLeft => {
            //Center of parent
            (parent_position + parent_dimension * 0.5) +
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
            (parent_position + parent_dimension) +
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
