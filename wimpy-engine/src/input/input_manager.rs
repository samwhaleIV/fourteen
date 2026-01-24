use smallvec::SmallVec;

use crate::input::{
    Direction,
    Impulse,
    ImpulseEvent,
    ImpulseState,
    InterpretiveAxes,
    KeyCode,
    KeyboardState,
    KeyboardTranslator,
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
    keyboard_state: KeyboardState,
    keyboard_translator: KeyboardTranslator,
    recent_input_method: InputType,
    impulse_state: ImpulseSet,
    last_directions: MoveToFrontStack<Direction,4>,
    recent_impulses: SmallVec<[ImpulseEvent;16]>,
}

pub trait InputManagerReadonly {
    fn get_gamepad_axes_or_default(&self) -> InterpretiveAxes;
    fn iter_recent_events(&self) -> impl Iterator<Item = &ImpulseEvent>;
    fn get_axes(&self) -> InterpretiveAxes;
    fn get_strict_direction(&self) -> Direction;
    fn get_active_input_type(&self) -> InputType;
}

pub trait InputManagerAppController {
    fn set_key_code_pressed(&mut self,key_code: KeyCode);
    fn set_key_code_released(&mut self,key_code: KeyCode);
    fn update(&mut self);
}

pub trait InputManagerBindController {
    fn clear_captured_key_code(&mut self);
    fn get_captured_key_code(&self) -> Option<KeyCode>;
    fn add_key_bind(&mut self,key_code: KeyCode,impulse: Impulse);
    fn remove_bind_for_key_code(&mut self,key_code: KeyCode);
    fn remove_binds_for_impulse(&mut self,impulse: Impulse);
    fn clear_all_key_binds(&mut self);
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
            keyboard_state: Default::default(),
            impulse_state: Default::default(),
            last_directions: Default::default(),
            recent_impulses: Default::default(),
            keyboard_translator: Default::default()
        }
    }
}

impl InputManagerReadonly for InputManager {
    fn get_gamepad_axes_or_default(&self) -> InterpretiveAxes {
        match &self.gamepad_manager {
            Some(value) => value.get_axes(),
            None => Default::default(),
        }
    }

    fn iter_recent_events(&self) -> impl Iterator<Item = &ImpulseEvent> {
        self.recent_impulses.iter()
    }

    fn get_axes(&self) -> InterpretiveAxes {
        match self.recent_input_method {
            InputType::Unknown | InputType::Keyboard => {
                self.impulse_state.get_axes()
            },
            InputType::Gamepad => {
                self.get_gamepad_axes_or_default()
            },
        }
    }

    fn get_strict_direction(&self) -> Direction {
        self.last_directions.peek()
    }

    fn get_active_input_type(&self) -> InputType {
        self.recent_input_method
    }
}

impl InputManagerBindController for InputManager {
    fn clear_captured_key_code(&mut self) {
        self.keyboard_state.clear_captured_key_code();
    }

    fn get_captured_key_code(&self) -> Option<KeyCode> {
        self.keyboard_state.get_captured_key_code()
    }
    
    fn add_key_bind(&mut self,key_code: KeyCode,impulse: Impulse) {
        self.keyboard_translator.add_key_bind(key_code,impulse);
        self.keyboard_state.release_all();
    }

    fn remove_bind_for_key_code(&mut self,key_code: KeyCode) {
        self.keyboard_translator.remove_bind_for_key_code(key_code);
        self.keyboard_state.release_all();
    }

    fn remove_binds_for_impulse(&mut self,impulse: Impulse) {
        self.keyboard_translator.remove_binds_for_impulse(impulse);
        self.keyboard_state.release_all();
    }

    fn clear_all_key_binds(&mut self) {
        self.keyboard_translator.clear_all_key_binds();
        self.keyboard_state.release_all();
    }
}

impl InputManagerAppController for InputManager {
    fn set_key_code_pressed(&mut self,key_code: KeyCode) {
        self.keyboard_state.set_pressed(key_code);
        self.recent_input_method = InputType::Keyboard;
        log::trace!("Key code pressed: {:?}",key_code);
    }

    fn set_key_code_released(&mut self,key_code: KeyCode) {
        self.keyboard_state.set_released(key_code);
        self.recent_input_method = InputType::Keyboard;
        log::trace!("Key code released: {:?}",key_code);
    }

    fn update(&mut self) {
        let keyboard_state = self.keyboard_translator.translate(&self.keyboard_state);

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
}
