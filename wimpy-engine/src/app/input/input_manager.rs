use smallvec::SmallVec;
use crate::collections::MoveToFrontStack;
use super::*;

// Should exceed the actual number of impulses!
const RECENT_IMPULSE_BUFFER_SIZE: usize = 16;

#[derive(Default)]
pub struct InputManager {
    recent_mouse_input:     mouse::MouseInput,
    gamepad_cache:          gamepad::GamepadCache,
    keyboard_state:         keyboard::KeyboardState,
    keyboard_translator:    keyboard::KeyboardTranslator,
    impulse_state:          ImpulseSet,
    last_directions:        MoveToFrontStack<Direction,4>,
    recent_impulses:        SmallVec<[ImpulseEvent;RECENT_IMPULSE_BUFFER_SIZE]>,
    captured_key_code:      Option<KeyCode>,
    virtual_mouse:          mouse::VirtualMouse,
    input_device_hint:      InputDevice,
    delta_seconds:          f32,
}

impl InputManager {
    pub fn with_device_start_hint(input_device_hint: InputDevice) -> Self {
        return Self {
            input_device_hint,
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
        let keyboard_axes = self.impulse_state.get_axes();

        return match keyboard_axes.is_zero() {
            false => keyboard_axes,
            true => self.gamepad_cache.left_axes(),
        };
    }

    pub fn get_virtual_mouse(&self) -> &mouse::VirtualMouse {
        &self.virtual_mouse
    }

    pub fn get_virtual_mouse_mut(&mut self) -> &mut mouse::VirtualMouse {
        &mut self.virtual_mouse
    }

    pub fn get_strict_direction(&self) -> Direction {
        self.last_directions.peek()
    }

    pub fn get_delta_seconds(&self) -> f32 {
        self.delta_seconds
    }
}

pub use key_rebind_controller::*;
pub use app_shell_controller::*;

mod key_rebind_controller {
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

mod app_shell_controller {
    use crate::WimpyRect;

    use super::*;
    impl InputManager {
        pub fn set_key_code_pressed(&mut self,key_code: KeyCode) {
            self.keyboard_state.set_pressed(key_code);
            self.input_device_hint = InputDevice::MouseAndKeyboard;
            if self.captured_key_code.is_none() {
                self.captured_key_code = Some(key_code);
            }
            // log::trace!("Key code pressed: {:?}",key_code);
        }

        pub fn set_key_code_released(&mut self,key_code: KeyCode) {
            self.keyboard_state.set_released(key_code);
            self.input_device_hint = InputDevice::MouseAndKeyboard;
            // log::trace!("Key code released: {:?}",key_code);
        }

        pub fn update(
            &mut self,
            mouse_input:    mouse::MouseInput,
            gamepad_input:  gamepad::GamepadInput,
            delta_seconds:  f32,
            bounds:         WimpyRect,
            can_reposition: bool,
        ) -> mouse::MouseShellState {
            let keyboard_state = self.keyboard_translator.translate(&self.keyboard_state);

            if self.gamepad_cache.update(gamepad_input) == UserActivity::Some {
                self.input_device_hint = InputDevice::Gamepad;
            }

            let old_mouse_input = &self.recent_mouse_input;
            if
                old_mouse_input.left_pressed != mouse_input.left_pressed ||
                old_mouse_input.right_pressed != mouse_input.right_pressed ||
                old_mouse_input.position != mouse_input.position
            {
                self.input_device_hint = InputDevice::MouseAndKeyboard;
            }
            self.recent_mouse_input = mouse_input;

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
                self.input_device_hint,
                self.recent_mouse_input,
                &self.gamepad_cache,
                delta_seconds,
                bounds,
                can_reposition
            );

            self.delta_seconds = delta_seconds;

            return virtual_mouse_shell_state;
        }
    }
}
