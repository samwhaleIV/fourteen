use super::prelude::*;

pub const IMPULSE_TYPE_COUNT: usize = 10;

pub const IMPULSES: [Impulse;IMPULSE_TYPE_COUNT] = [
    Impulse::Up,
    Impulse::Down,
    Impulse::Left,
    Impulse::Right,
    Impulse::Confirm,
    Impulse::Cancel,
    Impulse::FocusLeft,
    Impulse::FocusRight,
    Impulse::View,
    Impulse::Menu
];

#[derive(Clone,Copy,PartialEq,Eq,Debug)]
pub enum Impulse {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
    Confirm = 4,
    Cancel = 5,
    FocusLeft = 6,
    FocusRight = 7,
    View = 8,
    Menu = 9
}

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

#[derive(Clone,Copy,PartialEq,Eq,Debug)]
pub enum ImpulseState {
    Pressed,
    Released,
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

#[derive(Clone,Copy)]
pub struct ImpulseSet {
    actions: [ImpulseState;IMPULSE_TYPE_COUNT]
}

impl Default for ImpulseSet {
    fn default() -> Self {
        return Self {
            actions: array::repeat(ImpulseState::Released) 
        }
    }
}

#[derive(Clone,Copy,Debug)]
pub struct ImpulseEvent {
    pub impulse: Impulse,
    pub state: ImpulseState
}

pub struct ImpulseSetDescription {
    pub up: ImpulseState,
    pub down: ImpulseState,
    pub left: ImpulseState,
    pub right: ImpulseState,
    pub confirm: ImpulseState,
    pub cancel: ImpulseState,
    pub focus_left: ImpulseState,
    pub focus_right: ImpulseState,
    pub view: ImpulseState,
    pub menu: ImpulseState,
}

impl ImpulseSet {
    pub fn new(set: ImpulseSetDescription) -> Self {
        return Self {
            actions: [
                set.up,
                set.down,
                set.left,
                set.right,
                set.confirm,
                set.cancel,
                set.focus_left,
                set.focus_right,
                set.view,
                set.menu,
            ]
        }
    }

    pub fn get(&self,impulse: Impulse) -> ImpulseState {
        return self.actions[impulse as usize];
    }

    pub fn set(&mut self,impulse: Impulse,button_state: ImpulseState) {
        self.actions[impulse as usize] = button_state;
    }

    pub fn is_pressed(&self,impulse: Impulse) -> bool {
        return self.actions[impulse as usize] == ImpulseState::Pressed;
    }

    pub fn is_released(&self,impulse: Impulse) -> bool {
        return self.actions[impulse as usize] == ImpulseState::Released;
    }

    pub fn mix(a: &Self,b: &Self) -> Self {
        let mut impulse_set: Self = Default::default();
        for action in IMPULSES {
            impulse_set.set(action,match a.is_pressed(action) || b.is_pressed(action) {
                true => ImpulseState::Pressed,
                false => ImpulseState::Released,
            });
        }
        impulse_set
    }

    pub fn get_axes(&self) -> InterpretiveAxes {
        return InterpretiveAxes {
            x: InterpretiveAxis::from_impulse_state(
                self.get(Impulse::Left),
                self.get(Impulse::Right)
            ),
            y: InterpretiveAxis::from_impulse_state(
                self.get(Impulse::Up),
                self.get(Impulse::Down)
            ),
        };
    }

    pub fn iter_delta(&self,new: &ImpulseSet) -> impl Iterator<Item = ImpulseEvent> {
        IMPULSES.iter().filter_map(|&impulse| {
            let old_button_state = self.get(impulse);
            let new_button_state = new.get(impulse);
            (old_button_state != new_button_state).then_some(ImpulseEvent {
                impulse,
                state: new_button_state,
            })
        })
    }
}
