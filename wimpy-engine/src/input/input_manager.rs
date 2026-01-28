use smallvec::SmallVec;

use crate::input::{
    Direction, GamepadInput, Impulse, ImpulseEvent, ImpulseState, InterpretiveAxes, KeyCode, KeyboardState, KeyboardTranslator, UserActivity, gamepad::GamepadCache, impulse::ImpulseSet, move_to_front_stack::MoveToFrontStack
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

#[derive(Default)]
pub struct InputManager {
    gamepad_cache: GamepadCache,
    keyboard_state: KeyboardState,
    keyboard_translator: KeyboardTranslator,
    recent_input_method: InputType,
    impulse_state: ImpulseSet,
    last_directions: MoveToFrontStack<Direction,4>,
    recent_impulses: SmallVec<[ImpulseEvent;16]>,
    captured_key_code: Option<KeyCode>
}

impl InputManager {
    pub fn with_input_type_hint(input_type: InputType) -> Self {
        return Self {
            recent_input_method: input_type,
            ..Default::default()
        }
    }
}

pub trait InputManagerReadonly {
    fn iter_recent_events(&self) -> impl Iterator<Item = &ImpulseEvent>;
    fn get_axes(&self) -> InterpretiveAxes;
    fn get_strict_direction(&self) -> Direction;
    fn get_active_input_type(&self) -> InputType;
}

pub trait InputManagerAppController {
    fn set_key_code_pressed(&mut self,key_code: KeyCode);
    fn set_key_code_released(&mut self,key_code: KeyCode);
    fn update(&mut self,gamepad_input: GamepadInput);
}

pub trait InputManagerBindController {
    fn clear_captured_key_code(&mut self);
    fn get_captured_key_code(&self) -> Option<KeyCode>;
    fn add_key_bind(&mut self,key_code: KeyCode,impulse: Impulse);
    fn remove_bind_for_key_code(&mut self,key_code: KeyCode);
    fn remove_binds_for_impulse(&mut self,impulse: Impulse);
    fn clear_all_key_binds(&mut self);
}

impl InputManagerReadonly for InputManager {
    fn iter_recent_events(&self) -> impl Iterator<Item = &ImpulseEvent> {
        self.recent_impulses.iter()
    }

    fn get_axes(&self) -> InterpretiveAxes {
        match self.recent_input_method {
            InputType::Unknown | InputType::Keyboard => {
                self.impulse_state.get_axes()
            },
            InputType::Gamepad => {
                self.gamepad_cache.get_axes()
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
        self.captured_key_code = None;
    }

    fn get_captured_key_code(&self) -> Option<KeyCode> {
        return self.captured_key_code;
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
        if self.captured_key_code.is_none() {
            self.captured_key_code = Some(key_code);
        }
        log::trace!("Key code pressed: {:?}",key_code);
    }

    fn set_key_code_released(&mut self,key_code: KeyCode) {
        self.keyboard_state.set_released(key_code);
        self.recent_input_method = InputType::Keyboard;
        log::trace!("Key code released: {:?}",key_code);
    }

    fn update(&mut self,gamepad_input: GamepadInput) {
        let keyboard_state = self.keyboard_translator.translate(&self.keyboard_state);

        if self.gamepad_cache.update(gamepad_input) == UserActivity::Some {
            self.recent_input_method = InputType::Gamepad;
        }

        let gamepad_state = self.gamepad_cache.get_impulse_set();
        let new_state = ImpulseSet::mix(&keyboard_state,&gamepad_state);

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
