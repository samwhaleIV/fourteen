mod constants {
    pub const AXIS_DEADZONE: f32 = 0.15;
    pub const JOYSTICK_IMPULSE_THRESHOLD: f32 = 0.5; //TODO: have an ascending and descending value in case the change isn't monotonic
    pub const TRIGGER_IS_PRESSED_THREHOLD: f32 = 0.5;
    pub const JOYSTICK_CURSOR_PIXELS_PER_SECOND: f32 = 1500.0;
    pub const IMPULSE_COUNT: usize = super::IMPULSES.len();
}

mod input_manager;
pub use input_manager::InputManager;

mod gamepad;
pub use gamepad::*;

mod key_code;
pub use key_code::*;

mod keyboard;
pub use keyboard::*;

mod mouse;
pub use mouse::*;

// These enum integer values need to freeze once key binds are saved into a file. We aren't there yet.
#[derive(Clone,Copy,PartialEq,Eq,Debug)]
pub enum Impulse {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
    ZUp = 4,
    ZDown = 5,
    Confirm = 6,
    Cancel = 7,
    FocusLeft = 8,
    FocusRight = 9,
    View = 10,
    Menu = 11
}

pub const IMPULSES: [Impulse;12] = [
    Impulse::Up,
    Impulse::Down,
    Impulse::Left,
    Impulse::Right,
    Impulse::ZUp,
    Impulse::ZDown,
    Impulse::Confirm,
    Impulse::Cancel,
    Impulse::FocusLeft,
    Impulse::FocusRight,
    Impulse::View,
    Impulse::Menu
];

impl Impulse {
    pub fn direction(&self) -> Direction {
        match self {
            Impulse::Up =>      Direction::Up,
            Impulse::Down =>    Direction::Down,
            Impulse::Left =>    Direction::Left,
            Impulse::Right =>   Direction::Right,
            _ => Direction::None
        }
    }
}

#[derive(Default,Clone,Copy,PartialEq,Eq,Debug)]
pub enum ImpulseState {
    #[default]
    Released,
    Pressed,
}

impl From<ImpulseState> for bool {
    fn from(value: ImpulseState) -> Self {
        value == ImpulseState::Pressed
    }
}

impl From<bool> for ImpulseState {
    fn from(value: bool) -> Self {
        match value {
            true => ImpulseState::Pressed,
            false => ImpulseState::Released,
        }
    }
}

impl ImpulseState {
    pub fn from_bool(value: bool) -> Self {
        match value {
            true => Self::Pressed,
            false => Self::Released,
        }
    }
}

#[derive(Default,Clone,Copy)]
pub struct ImpulseSet {
    actions: [ImpulseState;constants::IMPULSE_COUNT]
}

#[derive(Clone,Copy,Debug)]
pub struct ImpulseEvent {
    pub impulse:    Impulse,
    pub state:      ImpulseState
}

pub struct ImpulseSetDescription {
    pub up:             ImpulseState,
    pub down:           ImpulseState,
    pub left:           ImpulseState,
    pub right:          ImpulseState,
    pub z_up:           ImpulseState,
    pub z_down:         ImpulseState,
    pub confirm:        ImpulseState,
    pub cancel:         ImpulseState,
    pub focus_left:     ImpulseState,
    pub focus_right:    ImpulseState,
    pub view:           ImpulseState,
    pub menu:           ImpulseState,
}

impl From<ImpulseSetDescription> for ImpulseSet {
    fn from(set: ImpulseSetDescription) -> Self {
        Self {
            actions: [
                set.up,
                set.down,
                set.left,
                set.right,
                set.z_up,
                set.z_down,
                set.confirm,
                set.cancel,
                set.focus_left,
                set.focus_right,
                set.view,
                set.menu,
            ]
        }
    }
}

impl ImpulseSet {
    pub fn get(&self,impulse: Impulse) -> ImpulseState {
        self.actions[impulse as usize]
    }

    pub fn set(&mut self,impulse: Impulse,button_state: ImpulseState) {
        self.actions[impulse as usize] = button_state;
    }

    pub fn is_pressed(&self,impulse: Impulse) -> bool {
        self.actions[impulse as usize] == ImpulseState::Pressed
    }

    pub fn is_released(&self,impulse: Impulse) -> bool {
        self.actions[impulse as usize] == ImpulseState::Released
    }

    pub fn mix(a: &Self,b: &Self) -> Self {
        let mut impulse_set: Self = Default::default();
        for action in IMPULSES.iter().copied() {
            impulse_set.set(action,match a.is_pressed(action) || b.is_pressed(action) {
                true => ImpulseState::Pressed,
                false => ImpulseState::Released,
            });
        }
        impulse_set
    }

    pub fn get_axes(&self) -> InterpretiveAxes {
        InterpretiveAxes {
            x: InterpretiveAxis::from_impulse_state(
                self.get(Impulse::Left),
                self.get(Impulse::Right)
            ),
            y: InterpretiveAxis::from_impulse_state(
                self.get(Impulse::Up),
                self.get(Impulse::Down)
            ),
        }
    }

    pub fn iter_delta(&self,new: &ImpulseSet) -> impl Iterator<Item = ImpulseEvent> {
        IMPULSES.iter().filter_map(|&impulse| {
            let old = self.get(impulse);
            let new = new.get(impulse);
            (old != new).then_some(ImpulseEvent {
                impulse,
                state: new,
            })
        })
    }
}

pub enum CardinalDirection {
    None, // Might want to get rid of none?
    North,
    South,
    West,
    East,
    NorthWest,
    NorthEast,
    SouthWest,
    SouthEast
}

#[derive(Default,Clone,Copy,PartialEq,Eq)]
pub enum Direction {
    #[default]
    None,
    Up,
    Down,
    Left,
    Right
}

impl Direction {
    pub fn sign(&self) -> AxisSign {
        match self {
            Direction::None =>  AxisSign::Zero,
            Direction::Up |     Direction::Left =>  AxisSign::Negative,
            Direction::Down |   Direction::Right => AxisSign::Positive,
        }
    }
}

#[derive(Default,Clone,Copy,PartialEq,Eq,Debug)]
pub enum AxisSign {
    #[default]
    Zero,
    Negative,
    Positive,
}

#[derive(Default,Clone,Copy,Debug)]
pub struct InterpretiveAxis {
    sign: AxisSign,
    value: f32
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
        use constants::AXIS_DEADZONE as deadzone;
        let out = if value.abs() <= deadzone {
            0.0
        } else {
            value.signum() * (value.abs() - deadzone) / (1.0 - deadzone)
        };
        Self {
            sign: get_axis_sign(out),
            value: out,
        }
    }

    pub fn from_bool(negative: bool,positive: bool) -> InterpretiveAxis {
        match (negative,positive) {
            (true, true) =>     InterpretiveAxis { sign: AxisSign::Zero,        value: 0.0 },
            (true, false) =>    InterpretiveAxis { sign: AxisSign::Negative,    value: -1.0 },
            (false, true) =>    InterpretiveAxis { sign: AxisSign::Positive,    value: 1.0 },
            (false, false) =>   InterpretiveAxis { sign: AxisSign::Zero,        value: 0.0 }
        }
    }

    pub fn from_impulse_state(negative: ImpulseState,positive: ImpulseState) -> InterpretiveAxis {
        match (negative,positive) {
            (ImpulseState::Pressed,     ImpulseState::Pressed) =>   InterpretiveAxis { sign: AxisSign::Zero,        value: 0.0 },
            (ImpulseState::Pressed,     ImpulseState::Released) =>  InterpretiveAxis { sign: AxisSign::Negative,    value: -1.0 },
            (ImpulseState::Released,    ImpulseState::Pressed) =>   InterpretiveAxis { sign: AxisSign::Positive,    value: 1.0 },
            (ImpulseState::Released,    ImpulseState::Released) =>  InterpretiveAxis { sign: AxisSign::Zero,        value: 0.0 }
        }
    }
}

impl From<InterpretiveAxis> for i32 {
    fn from(value: InterpretiveAxis) -> Self {
        match value.sign {
            AxisSign::Negative => -1,
            AxisSign::Zero => 0,
            AxisSign::Positive => 1,
        }
    }
}

impl From<InterpretiveAxis> for f32 {
    fn from(value: InterpretiveAxis) -> Self {
        value.value
    }
}

impl From<InterpretiveAxes> for crate::WimpyVec {
    fn from(value: InterpretiveAxes) -> Self {
        Self {
            x: value.x.value,
            y: value.y.value,
        }
    }
}

#[derive(Default,Copy,Clone,Debug)]
pub struct InterpretiveAxes {
    pub x: InterpretiveAxis,
    pub y: InterpretiveAxis
}

impl InterpretiveAxes {
    pub fn x(&self) -> f32 {
        self.x.value
    }

    pub fn y(&self) -> f32 {
        self.y.value
    }

    pub fn is_zero(&self) -> bool {
        self.x.sign == AxisSign::Zero &&
        self.y.sign == AxisSign::Zero
    }

    pub fn infer_impulse(&self,direction: Direction,threshold: f32) -> ImpulseState {
        let axis = match direction {
            Direction::Left |   Direction::Right =>     &self.x,
            Direction::Up |     Direction::Down =>      &self.y,
            Direction::None =>  return ImpulseState::Released,
        };
        match (
            axis.sign == direction.sign(),
            axis.value.abs() >= threshold
        ) {
            (true,true) =>  ImpulseState::Pressed,
            _ =>            ImpulseState::Released
        }
    }

    pub fn get_cardinal_direction(&self) -> CardinalDirection {
        match (self.x.sign,self.y.sign) {
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

#[derive(Default,Copy,Clone)]
pub struct InterpetiveTrigger {
    value: f32,
    is_pressed: bool,
}

impl InterpetiveTrigger {
    fn create(value: f32) -> Self {
        Self {
            value,
            is_pressed: value >= constants::TRIGGER_IS_PRESSED_THREHOLD
        }
    }
    pub fn is_pressed(&self) -> bool {
        self.is_pressed
    }
    pub fn value(&self) -> f32 {
        self.value
    }
}

#[derive(PartialEq,Eq)]
enum UserActivity {
    None,
    Some
}

// Do not use for input control flow! Only for UI hints
#[derive(Debug,Default,Copy,Clone,PartialEq,Eq)]
pub enum InputDevice {
    #[default]
    MouseAndKeyboard,
    Gamepad
}
