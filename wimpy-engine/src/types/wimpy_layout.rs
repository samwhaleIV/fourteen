use super::*;

#[derive(Default,Copy,Clone)]
pub struct WimpyLayout {
    pub x: LayoutDimension,
    pub y: LayoutDimension
}

#[derive(Default,Copy,Clone)]
pub enum Align {
    #[default]
    Absolute,
    LeftToRight,
    RightToLeft,
    Center,
    CenterLeftToRight,
    CenterRightToLeft,
}

#[derive(Default,Copy,Clone)]
pub enum SizeMode {
    #[default]
    Absolute,
    RelativeWidth,
    RelativeHeight,
    RelativeSmallest,
    RelativeLargest
}

#[derive(Default,Copy,Clone)]
pub struct Position {
    pub value: Size,
    pub alignment: Align
}

impl Position {
    pub fn center_of_parent() -> Self {
        Self {
            value: Size::default(),
            alignment: Align::Center
        }
    }
    pub fn center_of_parent_with_offset(size: Size) -> Self {
        Self {
            value: size,
            alignment: Align::Center
        }
    }
}

#[derive(Default,Copy,Clone)]
pub struct Size {
    pub value: f32,
    pub mode: SizeMode
}

impl From<f32> for Size {
    fn from(value: f32) -> Self {
        Self {
            value,
            mode: SizeMode::Absolute,
        }
    }
}

impl Size {
    pub fn of_parent_height(value: f32) -> Self {
        Self {
            value,
            mode: SizeMode::RelativeHeight,
        }
    }
    pub fn of_parent_width(value: f32) -> Self {
        Self {
            value,
            mode: SizeMode::RelativeWidth,
        }
    }
    pub fn of_parent_smallest(value: f32) -> Self {
        Self {
            value,
            mode: SizeMode::RelativeSmallest,
        }
    }
    pub fn of_parent_largest(value: f32) -> Self {
        Self {
            value,
            mode: SizeMode::RelativeLargest,
        }
    }
}

#[derive(Default,Copy,Clone)]
pub struct LayoutDimension {
    pub position: Position,
    pub size: Size,
    pub size_offset: Size,
}

impl WimpyLayout {
    //Top Left Encoded
    pub fn compute(&self,parent: WimpyRect) -> WimpyRect {  
        let (x,width) = calc_layout_dim(
            self.x,parent.x(),
            parent.width(),
            parent.size
        );
        let (y,height) = calc_layout_dim(
            self.y,parent.y(),
            parent.height(),
            parent.size
        );
        WimpyRect::from([x,y,width,height])
    }
}

impl From<LayoutDimension> for WimpyLayout {
    fn from(value: LayoutDimension) -> Self {
        Self {
            x: value,
            y: value
        }
    }
}

fn calc_layout_dim(
    dim: LayoutDimension,
    p_pos: f32,
    p_len: f32,
    p_size: WimpyVec,
) -> (f32,f32) {
    let mut len = calc_len(dim.size,p_size);
    let mut pos = calc_pos(dim.position,len,p_pos,p_len,p_size);

    /* Applies after all other layout calculation. */
    let ofs = calc_len(dim.size_offset,p_size);

    /* Inset or outset position based on the size change */
    pos += ofs * -0.5;
    len += ofs;

    (pos,len)
}

fn calc_len(
    size: Size,
    p_size: WimpyVec,
) -> f32 {
    match size.mode {
        SizeMode::Absolute => {
            size.value
        },
        SizeMode::RelativeWidth => {
            p_size.x * size.value
        },
        SizeMode::RelativeHeight => {
            p_size.y * size.value
        },
        SizeMode::RelativeSmallest => {
            p_size.x.min(p_size.y) * size.value
        },
        SizeMode::RelativeLargest => {
            p_size.x.max(p_size.y) * size.value
        },
    }
}

fn calc_pos(
    pos: Position,
    c_len: f32,
    p_pos: f32,
    p_len: f32,
    p_size: WimpyVec,
) -> f32 {
    let p_ofs = calc_len(pos.value,p_size);

    match pos.alignment {
        Align::Center => {
            //Translate to center of parent
            (p_pos + p_len * 0.5) +
            //Align child on axis line
            (c_len * -0.5) +
            //Apply offset in regular LTR
            p_ofs
        },
        
        Align::CenterLeftToRight => {
            //Translate to center of parent
            (p_pos + p_len * 0.5) +
            //Apply offset in regular LTR
            p_ofs
        },

        Align::CenterRightToLeft => {
            //Center of parent
            (p_pos + p_len * 0.5) +
            //Push right edge to axis line
            (c_len * -1.0) +
            //Apply offset inverted because of RTL
            (p_ofs * -1.0)
        },

        Align::LeftToRight => {
            //Parent position, which we inherit
            p_pos +
            //Apply offset in regular LTR
            p_ofs
        },

        Align::RightToLeft => {
            //Translate to center of parent
            (p_pos + p_len) +
            //Push right edge to axis line
            (c_len * -1.0) +
            //Apply offset inverted because of RTL
            (p_ofs * -1.0)
        },

        /* Position is absolute, but the value itself can be parent size relative. */
        Align::Absolute => {
            //Apply offset in regular LTR. No constraint to parent bound
            p_ofs
        },
    }
}

impl From<[WimpyVec;2]> for WimpyRect {
    fn from(value: [WimpyVec;2]) -> Self {
        Self {
            position: value[0],
            size: value[1],
        }
    }
}

impl From<[u32;4]> for WimpyRect {
    fn from(value: [u32;4]) -> Self {
        Self {
            position: WimpyVec {
                x: value[0] as f32,
                y: value[1] as f32,
            },
            size: WimpyVec {
                x: value[2] as f32,
                y: value[3] as f32
            },
        }
    }
}

impl From<[i32;4]> for WimpyRect {
    fn from(value: [i32;4]) -> Self {
        Self {
            position: WimpyVec {
                x: value[0] as f32,
                y: value[1] as f32,
            },
            size: WimpyVec {
                x: value[2] as f32,
                y: value[3] as f32
            },
        }
    }
}

impl From<[f32;4]> for WimpyRect {
    fn from(value: [f32;4]) -> Self {
        Self {
            position: WimpyVec {
                x: value[0],
                y: value[1],
            },
            size: WimpyVec {
                x: value[2],
                y: value[3]
            },
        }
    }
}
