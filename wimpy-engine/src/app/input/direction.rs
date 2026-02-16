use super::prelude::*;

pub enum CardinalDirection {
    None,
    North,
    South,
    West,
    East,
    NorthWest,
    NorthEast,
    SouthWest,
    SouthEast
}

#[derive(Clone,Copy,PartialEq,Eq)]
pub enum Direction {
    None,
    Up,
    Down,
    Left,
    Right
}

impl Direction {
    pub fn sign(&self) -> AxisSign {
        match self {
            Direction::None => AxisSign::Zero,
            Direction::Up | Direction::Left => AxisSign::Negative,
            Direction::Down | Direction::Right => AxisSign::Positive,
        }
    }
}

impl Default for Direction {
    fn default() -> Self {
        return Self::None;
    }
}

#[derive(Clone,Copy,PartialEq,Eq,Debug)]
pub enum AxisSign {
    Negative,
    Zero,
    Positive,
}

#[derive(Clone,Copy,Debug)]
pub struct InterpretiveAxis {
    sign: AxisSign,
    value: f32
}

impl Default for InterpretiveAxis {
    fn default() -> Self {
        Self {
            sign: AxisSign::Zero,
            value: 0.0
        }
    }
}

fn get_axis_sign(value: f32) -> AxisSign {
    match value {
        _ if value > 0.0 => AxisSign::Positive,
        _ if value < 0.0 => AxisSign::Negative,
        _ => AxisSign::Zero
    }
}

impl InterpretiveAxis {
    pub fn from_f32_with_deadzone(value: f32) -> Self {
        let out = if value.abs() <= AXIS_DEADZONE {
            0.0
        } else {
            value.signum() * (value.abs() - AXIS_DEADZONE) / (1.0 - AXIS_DEADZONE)
        };
        Self {
            sign: get_axis_sign(out),
            value: out,
        }
    }
    pub fn from_bool(negative: bool,positive: bool) -> InterpretiveAxis {
        match (negative,positive) {
            (true, true) => InterpretiveAxis {
                sign: AxisSign::Zero,
                value: 0.0,
            },
            (true, false) => InterpretiveAxis {
                sign: AxisSign::Negative,
                value: -1.0,
            },
            (false, true) => InterpretiveAxis {
                sign: AxisSign::Positive,
                value: 1.0,
            },
            (false, false) => InterpretiveAxis {
                sign: AxisSign::Zero,
                value: 0.0,
            }
        }
    }
    pub fn from_impulse_state(negative: ImpulseState,positive: ImpulseState) -> InterpretiveAxis {
        match (negative,positive) {
            (ImpulseState::Pressed,     ImpulseState::Pressed) => InterpretiveAxis {
                sign: AxisSign::Zero,
                value: 0.0,
            },
            (ImpulseState::Pressed,     ImpulseState::Released) => InterpretiveAxis {
                sign: AxisSign::Negative,
                value: -1.0,
            },
            (ImpulseState::Released,    ImpulseState::Pressed) => InterpretiveAxis {
                sign: AxisSign::Positive,
                value: 1.0,
            },
            (ImpulseState::Released,    ImpulseState::Released) => InterpretiveAxis {
                sign: AxisSign::Zero,
                value: 0.0,
            }
        }
    }
    pub fn get_i32(&self) -> i32 {
        match self.sign {
            AxisSign::Negative => -1,
            AxisSign::Zero => 0,
            AxisSign::Positive => 1,
        }
    }
    pub fn get_f32(&self) -> f32 {
        self.value
    }
}

#[derive(Default,Copy,Clone,Debug)]
pub struct InterpretiveAxes {
    x: InterpretiveAxis,
    y: InterpretiveAxis
}

impl InterpretiveAxes {
    pub fn create(x: InterpretiveAxis,y: InterpretiveAxis) -> Self {
        return Self {
            x,
            y,
        }
    }

    pub fn get_f32(&self) -> (f32,f32) {
        return (self.x.get_f32(),self.y.get_f32());
    }

    pub fn get_x_f32(&self) -> f32 {
        return self.x.get_f32();
    }

    pub fn get_y_f32(&self) -> f32 {
        return self.y.get_f32();
    }

    pub fn get_x_i32(&self) -> i32 {
        return self.x.get_i32();
    }

    pub fn get_y_i32(&self) -> i32 {
        return self.y.get_i32();
    }

    pub fn get_i32(&self) -> (i32,i32) {
        return (self.x.get_i32(),self.y.get_i32());
    }

    pub fn is_zero(&self) -> bool {
        return self.x.sign == AxisSign::Zero && self.y.sign == AxisSign::Zero;
    }

    pub fn infer_impulse(&self,direction: Direction,threshold: f32) -> ImpulseState {
        let axis = match direction {
            Direction::Left |   Direction::Right => &self.x,
            Direction::Up |     Direction::Down =>  &self.y,
            Direction::None => return ImpulseState::Released,
        };
        match (
            axis.sign == direction.sign(),
            axis.value.abs() >= threshold
        ) {
            (true,true) => ImpulseState::Pressed,
            _ => ImpulseState::Released
        }
    }

    pub fn get_cardinal_direction(&self) -> CardinalDirection {
        return match (self.x.sign,self.y.sign) {
            (AxisSign::Negative,    AxisSign::Negative) =>  CardinalDirection::NorthWest,
            (AxisSign::Negative,    AxisSign::Zero) =>      CardinalDirection::West,
            (AxisSign::Negative,    AxisSign::Positive) =>  CardinalDirection::SouthWest,
            (AxisSign::Zero,        AxisSign::Negative) =>  CardinalDirection::North,
            (AxisSign::Zero,        AxisSign::Zero) =>      CardinalDirection::None,
            (AxisSign::Zero,        AxisSign::Positive) =>  CardinalDirection::South,
            (AxisSign::Positive,    AxisSign::Negative) =>  CardinalDirection::NorthEast,
            (AxisSign::Positive,    AxisSign::Zero) =>      CardinalDirection::East,
            (AxisSign::Positive,    AxisSign::Positive) =>  CardinalDirection::SouthEast,
        }
    }
}
