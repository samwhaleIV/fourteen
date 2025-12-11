pub struct Area {
    pub top:    f32,
    pub left:   f32,
    pub right:  f32,
    pub bottom: f32,
}

impl Area {

    pub const NORMAL: Self = Self {
        top:    0.0,
        left:   0.0,
        right:  1.0,
        bottom: 1.0,
    };

    pub const INVERT_X: Self = Self {
        top:    0.0,
        left:   1.0,
        right:  0.0,
        bottom: 1.0,
    };

    pub const INVERT_Y: Self = Self {
        top:    1.0,
        left:   0.0,
        right:  1.0,
        bottom: 0.0,
    };

    pub const INVERT_XY: Self = Self {
        top:    1.0,
        left:   1.0,
        right:  0.0,
        bottom: 0.0,
    };

    pub fn centered_on(x: f32,y: f32,width: f32,height: f32) -> Area {
        let half_width = width * 0.5;
        let half_height = height * 0.5;
        return Area {
            top: y - half_height,
            left: x - half_width,
            right: x + half_width,
            bottom: y + half_height
        };
    }

    pub fn width(&self) -> f32 {
        return self.right - self.left;
    }

    pub fn height(&self) -> f32 {
        return self.bottom - self.top;
    }

    pub fn size(&self) -> (f32,f32) {
        return (self.width(),self.height());
    }

    pub fn center(&self) -> (f32,f32) {
        let (width,height) = self.size();
        let x = self.left + width * 0.5;
        let y = self.top + height * 0.5;
        return (x,y);
    }

    pub fn top_left(&self) -> (f32,f32) {
        return (self.left,self.top);
    }

    pub fn bottom_right(&self) -> (f32,f32) {
        return (self.right,self.bottom);
    }

    pub fn from_size(x: f32,y: f32,width: f32,height: f32) -> Area {
        return Area {
            top: y,
            left: x,
            right: x + width,
            bottom: y + height
        };
    }
}
