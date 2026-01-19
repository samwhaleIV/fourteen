use smallvec::SmallVec;

use crate::input::{
    Direction,
    ImpulseEvent,
    ImpulseState,
    InterpretiveAxes,
    UserActivity,
    gamepad_manager::{
        GamepadManager,
        GamepadManagerError
    },
    impulse::ImpulseSet,
    move_to_front_stack::MoveToFrontStack
};

#[derive(Copy,Clone)]
pub enum InputType {
    Unknown,
    Keyboard,
    Gamepad
}

impl Default for InputType {
    fn default() -> Self {
        return Self::Unknown;
    }
}

pub struct InputManager {
    gamepad_manager: Option<GamepadManager>,
    recent_input_method: InputType,
    impulse_state: ImpulseSet,
    last_directions: MoveToFrontStack<Direction,4>,
    recent_impulses: SmallVec<[ImpulseEvent;16]>,
}

impl Default for InputManager {
    fn default() -> Self {
        let gamepad_manager = match GamepadManager::new() {
            Ok(value) => Some(value),
            Err(error) => {
                log::warn!("{}",match error {
                    GamepadManagerError::UnsupportedGilrsPlatform => {
                        "Unsupported gilrs platform; application running without gamepad support."
                    },
                    GamepadManagerError::UnknownGilrsError => {
                        "Could not initialize gilrs; application running without gamepad support."
                    },
                });
                None
            },
        };
        Self {
            gamepad_manager,
            recent_input_method: Default::default(),
            impulse_state: Default::default(),
            last_directions: Default::default(),
            recent_impulses: Default::default(),
            
        }
    }
}

impl InputManager {
    pub fn update(&mut self,keyboard_state: ImpulseSet) {
        let new_state =  match &mut self.gamepad_manager {
            Some(gamepad_manager) => {
                let gamepad_activity = gamepad_manager.update();
                if gamepad_activity == UserActivity::Some {
                    self.recent_input_method = InputType::Gamepad;
                }
                let gamepad_state = gamepad_manager.get_impulse_set();
                ImpulseSet::mix(&keyboard_state,&gamepad_state)
            },
            None => keyboard_state,
        };

        self.recent_impulses.clear();

        for event in self.impulse_state.iter_delta(&new_state) {
            let direction = event.impulse.direction();
            match direction {
                Direction::None => {},
                _ => match event.state {
                    ImpulseState::Pressed => self.last_directions.push(direction),
                    ImpulseState::Released => self.last_directions.remove(direction),
                },
            };

            self.recent_impulses.push(event);
        }

        self.impulse_state = new_state;
    }

    fn get_gamepad_axes_or_default(&self) -> InterpretiveAxes {
        match &self.gamepad_manager {
            Some(value) => value.get_axes(),
            None => Default::default(),
        }
    }

    pub fn iter_recent_events(&self) -> impl Iterator<Item = ImpulseEvent> {
        self.recent_impulses.iter().copied()
    }

    pub fn get_axes(&self) -> InterpretiveAxes {
        match self.recent_input_method {
            InputType::Unknown | InputType::Keyboard => {
                self.impulse_state.get_axes()
            },
            InputType::Gamepad => {
                self.get_gamepad_axes_or_default()
            },
        }
    }

    pub fn get_strict_direction(&self) -> Direction {
        self.last_directions.peek()
    }

    pub fn get_active_input_type(&self) -> InputType {
        self.recent_input_method
    }
}
