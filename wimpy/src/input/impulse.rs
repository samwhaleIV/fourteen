use std::array;

use crate::input::{
    Direction,
    InterpretiveAxes,
    InterpretiveAxis,
};

pub const IMPULSE_TYPE_COUNT: usize = 8;

pub const IMPULSES: [Impulse;IMPULSE_TYPE_COUNT] = [
    Impulse::Up,
    Impulse::Down,
    Impulse::Left,
    Impulse::Right,
    Impulse::Confirm,
    Impulse::Back,
    Impulse::Focus,
    Impulse::Context
];

#[derive(Clone,Copy,PartialEq,Eq)]
pub enum Impulse {
    Up,
    Down,
    Left,
    Right,
    Confirm,
    Back,
    Focus,
    Context
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

#[derive(Clone,Copy,PartialEq,Eq)]
pub enum ImpulseState {
    Pressed,
    Released,
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

#[derive(Clone,Copy)]
pub struct ImpulseEvent {
    pub impulse: Impulse,
    pub state: ImpulseState
}

impl ImpulseSet {
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
