use smallvec::SmallVec;
use crate::collections::MoveToFrontStack;
use super::prelude::*;

// Should exceed the actual number of impulses!
const RECENT_IMPULSE_BUFFER_SIZE: usize = 16;

#[derive(PartialEq,Eq)]
pub enum UserActivity {
    None,
    Some
}

#[derive(Default,Copy,Clone)]
pub enum InputType {
    #[default]
    Unknown,
    KeyboardAndMouse,
    Gamepad
}

#[derive(Default)]
pub struct InputManager {
    previous_mouse_state: MouseInput,
    gamepad_cache: GamepadCache,
    keyboard_state: KeyboardState,
    keyboard_translator: KeyboardTranslator,
    recent_input_method: InputType,
    impulse_state: ImpulseSet,
    last_directions: MoveToFrontStack<Direction,4>,
    recent_impulses: SmallVec<[ImpulseEvent;RECENT_IMPULSE_BUFFER_SIZE]>,
    captured_key_code: Option<KeyCode>,
    virtual_mouse: VirtualMouse,
    delta_seconds: f32
}

impl InputManager {
    pub fn with_input_type_hint(input_type: InputType) -> Self {
        return Self {
            recent_input_method: input_type,
            ..Default::default()
        }
    }

    pub fn iter_recent_events(&self) -> impl Iterator<Item = &ImpulseEvent> {
        self.recent_impulses.iter()
    }

    pub fn get_state(&self,impulse: Impulse) -> ImpulseState {
        self.impulse_state.get(impulse)
    }

    pub fn is_pressed(&self,impulse: Impulse) -> bool {
        match self.impulse_state.get(impulse) {
            ImpulseState::Pressed => true,
            ImpulseState::Released => false,
        }
    }

    pub fn is_released(&self,impulse: Impulse) -> bool {
        match self.impulse_state.get(impulse) {
            ImpulseState::Pressed => false,
            ImpulseState::Released => true,
        }
    }

    pub fn get_axes(&self) -> InterpretiveAxes {
        match self.recent_input_method {
            InputType::Unknown | InputType::KeyboardAndMouse => {
                self.impulse_state.get_axes()
            },
            InputType::Gamepad => {
                self.gamepad_cache.left_axes()
            },
        }
    }

    pub fn get_virtual_mouse(&self) -> &VirtualMouse {
        &self.virtual_mouse
    }

    pub fn get_virtual_mouse_mut(&mut self) -> &mut VirtualMouse {
        &mut self.virtual_mouse
    }

    pub fn get_strict_direction(&self) -> Direction {
        self.last_directions.peek()
    }

    pub fn get_active_input_type(&self) -> InputType {
        self.recent_input_method
    }

    pub fn get_delta_seconds(&self) -> f32 {
        self.delta_seconds
    }
}

pub mod key_rebind_controller {
    use super::*;
    impl InputManager {
        pub fn clear_captured_key_code(&mut self) {
            self.captured_key_code = None;
        }

        pub fn get_captured_key_code(&self) -> Option<KeyCode> {
            return self.captured_key_code;
        }
        
        pub fn add_key_bind(&mut self,key_code: KeyCode,impulse: Impulse) {
            self.keyboard_translator.add_key_bind(key_code,impulse);
            self.keyboard_state.release_all();
        }

        pub fn remove_bind_for_key_code(&mut self,key_code: KeyCode) {
            self.keyboard_translator.remove_bind_for_key_code(key_code);
            self.keyboard_state.release_all();
        }

        pub fn remove_binds_for_impulse(&mut self,impulse: Impulse) {
            self.keyboard_translator.remove_binds_for_impulse(impulse);
            self.keyboard_state.release_all();
        }

        pub fn clear_all_key_binds(&mut self) {
            self.keyboard_translator.clear_all_key_binds();
            self.keyboard_state.release_all();
        }
    }
}

pub mod app_shell_controller {
    use crate::shared::WimpyArea;

    use super::*;
    impl InputManager {
        pub fn set_key_code_pressed(&mut self,key_code: KeyCode) {
            self.keyboard_state.set_pressed(key_code);
            self.recent_input_method = InputType::KeyboardAndMouse;
            if self.captured_key_code.is_none() {
                self.captured_key_code = Some(key_code);
            }
            log::trace!("Key code pressed: {:?}",key_code);
        }

        pub fn set_key_code_released(&mut self,key_code: KeyCode) {
            self.keyboard_state.set_released(key_code);
            self.recent_input_method = InputType::KeyboardAndMouse;
            log::trace!("Key code released: {:?}",key_code);
        }

        pub fn update(
            &mut self,
            mouse_input: MouseInput,
            gamepad_input: GamepadInput,
            delta_seconds: f32,
            mouse_emulation_bounds: WimpyArea,
        ) -> VirtualMouseShellState {
            let keyboard_state = self.keyboard_translator.translate(&self.keyboard_state);

            if mouse_input.has_significant_activity(&self.previous_mouse_state) {
                self.recent_input_method = InputType::KeyboardAndMouse;
            }
            self.previous_mouse_state = mouse_input;

            if self.gamepad_cache.update(gamepad_input) == UserActivity::Some {
                self.recent_input_method = InputType::Gamepad;
            }

            let gamepad_state = self.gamepad_cache.impulse_set();
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

            let virtual_mouse_shell_state = self.virtual_mouse.update(
                &self.previous_mouse_state,
                &self.gamepad_cache,
                self.recent_input_method,
                delta_seconds,
                mouse_emulation_bounds,
            );

            self.delta_seconds = delta_seconds;

            return virtual_mouse_shell_state;
        }
    }
}
