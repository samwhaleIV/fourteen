use crate::shared::WimpyArea;
use super::prelude::*;

const JOYSTICK_CURSOR_PIXELS_PER_SECOND: f32 = 1500.0;

#[derive(Default,Copy,Clone,PartialEq,Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32
}

impl Position {
    fn add_delta(mut self,delta: &Delta) -> Self {
        self.x += delta.x;
        self.y += delta.y;
        self
    }
    fn clip(mut self,bounds: WimpyArea) -> Self {
        let point = bounds.clip(self.x,self.y);
        self.x = point.0;
        self.y = point.1;
        self
    }
}

#[derive(Default,Copy,Clone,PartialEq,Debug)]
pub struct Delta {
    pub x: f32,
    pub y: f32
}

#[derive(Default,PartialEq,Copy,Clone)]
pub struct MouseInput {
    pub position: Position,
    pub delta: Delta,
    pub left_pressed: bool,
    pub right_pressed: bool,
}

#[derive(Clone,Copy)]
pub enum MouseModeSwitchCommand {
    InterfaceToCamera,
    CameraToInterface,
}

impl MouseInput {
    fn from_gamepad(gamepad: &GamepadCache,position: Position,retain_position: bool,emulation_bounds: WimpyArea,delta_seconds: f32) -> Self {
        let max_pixels = JOYSTICK_CURSOR_PIXELS_PER_SECOND * delta_seconds;
        let delta = Delta {
            x: gamepad.right_axes().get_x_f32() * max_pixels,
            y: gamepad.right_axes().get_y_f32() * max_pixels
        };
        let position = match retain_position {
            true => position,
            false => position.add_delta(&delta).clip(emulation_bounds),
        };
        return Self {
            position,
            delta,
            left_pressed: gamepad.left_trigger().is_pressed(),
            right_pressed: gamepad.right_trigger().is_pressed(),
        }
    }
    fn position_differs(&self,other: &Self) -> bool {
        self.position != other.position ||
        self.delta != other.delta
    }
    fn create_fused(&self,other: &Self) -> Self {
        return Self {
            position: self.position,
            delta: self.delta,
            left_pressed: self.left_pressed || other.left_pressed,
            right_pressed: self.right_pressed || other.right_pressed,
        }
    }
    fn has_delta_activity(&self) -> bool {
        return self.delta.x != 0.0 || self.delta.y != 0.0;
    }
}

#[derive(Default)]
pub struct VirtualMouse {
    fused_state: MouseInput,

    mouse_state: MouseInput,
    gamepad_state: MouseInput,

    left_press_state: MousePressState,
    right_press_state: MousePressState,

    previous_mode: MouseMode,
    current_mode: MouseMode,

    interaction_state: InteractionState,
    emulation_active: bool,

    hide_emulated_cursor_over_ui: bool,
    hide_camera_center_crosshair: bool,

    gamepad_position_init: bool
}

#[derive(Default,Copy,Clone,PartialEq,Eq)]
pub enum MousePressState {
    #[default]
    Released,
    JustPressed,
    Pressed,
    JustReleased,
}

impl From<MousePressState> for bool {
    fn from(value: MousePressState) -> Self {
        match value {
            MousePressState::Released => false,
            MousePressState::JustPressed => true,
            MousePressState::Pressed => true,
            MousePressState::JustReleased => false,
        }
    }
}

#[derive(Default,Copy,Clone,PartialEq,Eq)]
pub enum CursorGlyph {
    None,
    #[default]
    Default,
    CanInteract,
    IsInteracting,
    CameraCrosshair,
}

#[derive(Default,Copy,Clone)]
pub enum InteractionState {
    #[default]
    Default,
    Hidden,
    CanInteract,
    IsInteracting
}

#[derive(Debug,Default,Copy,Clone,PartialEq,Eq)]
pub enum MouseMode {
    #[default]
    Interface,
    Camera
}

#[derive(Debug,Default,Copy,Clone,PartialEq,Eq)]
pub enum CursorRenderingStrategy {
    #[default]
    Hardware,
    Emulated
}

impl From<InteractionState> for CursorGlyph {
    fn from(value: InteractionState) -> Self {
        match value {
            InteractionState::Hidden => Self::None,
            InteractionState::Default => Self::Default,
            InteractionState::CanInteract => Self::CanInteract,
            InteractionState::IsInteracting => Self::IsInteracting,
        }
    }
}

pub struct VirtualMouseShellState {
    pub cursor_glyph: CursorGlyph,
    pub position: Position,
    pub cursor_rendering_strategy: CursorRenderingStrategy,
    pub mouse_mode: MouseMode,
}

fn get_delta_press_state(old_state: MousePressState,is_pressed: bool) -> MousePressState {
    use MousePressState::*;
    match (old_state,is_pressed) {
        (Released, true) => JustPressed,
        (Released, false) => Released,
        (JustPressed, true) => Pressed,
        (JustPressed, false) => JustReleased,
        (Pressed, true) => Pressed,
        (Pressed, false) => JustReleased,
        (JustReleased, true) => JustPressed,
        (JustReleased, false) => Released,
    }
}

impl VirtualMouse {
    pub(super) fn update(
        &mut self,
        input_hint: InputType,
        new_mouse_state: MouseInput,
        gamepad: &GamepadCache,
        delta_seconds: f32,
        emulation_bounds: WimpyArea,
    ) -> VirtualMouseShellState {

        if !self.gamepad_position_init {
            let center = emulation_bounds.center();
            self.gamepad_position_init = true;
            self.gamepad_state.position = Position {
                x: center.0,
                y: center.1
            };
        }

        self.gamepad_state = MouseInput::from_gamepad(
            gamepad,
            self.gamepad_state.position,
            self.current_mode == MouseMode::Camera,
            emulation_bounds,
            delta_seconds
        );

        let previous_mouse_state = self.mouse_state;
        self.mouse_state = new_mouse_state;

        if self.previous_mode != self.current_mode {
            self.mouse_state.delta = Delta::default();
            self.gamepad_state.delta = Delta::default();
            match input_hint {
                InputType::Unknown | InputType::Keyboard => {
                    self.emulation_active = false;
                },
                InputType::Gamepad => {
                    self.emulation_active = true;
                },
            };
        }
        self.previous_mode = self.current_mode;

        match (
            self.gamepad_state.has_delta_activity(),
            self.mouse_state.position_differs(&previous_mouse_state)
        ) {
            /* Mouse control */
            (false, true) => {
                self.fused_state = self.mouse_state.create_fused(&self.gamepad_state);
                // if self.emulation_active {
                //     self.fused_state.delta = Delta::default();
                // }
                self.emulation_active = false;
            },
            /* Gamepad control */
            (true, false) => {
                self.fused_state = self.gamepad_state.create_fused(&self.mouse_state);
                // if !self.emulation_active {
                //     self.fused_state.delta = Delta::default();
                // }
                self.emulation_active = true;
            },
            /* Reuse previous frame's priority */
            (true, true) | (false, false) => match self.emulation_active {
                true => {
                    // Mouse buttons on top of gamepad axis
                    self.fused_state = self.gamepad_state.create_fused(&self.mouse_state);
                },
                false => {
                    // Gamepad buttons on top of mouse movement
                    self.fused_state = self.mouse_state.create_fused(&self.gamepad_state);
                },
            },
        }

        self.left_press_state = get_delta_press_state(
            self.left_press_state,
            self.fused_state.left_pressed
        );

        self.right_press_state = get_delta_press_state(
            self.right_press_state,
            self.fused_state.right_pressed
        );

        return self.get_cursor_shell_state();
    }

    fn get_cursor_shell_state(&self) -> VirtualMouseShellState {

        let glyph: CursorGlyph;
        let strategy: CursorRenderingStrategy;

        match self.current_mode {
            MouseMode::Camera => {
                glyph = match self.hide_camera_center_crosshair {
                    false => CursorGlyph::CameraCrosshair,
                    true => CursorGlyph::None,
                };
                // Even though the cursor won't be visible, we prime the cursor rendering in anticipation of a mode swap
                strategy = match self.emulation_active {
                    true => CursorRenderingStrategy::Emulated,
                    false => CursorRenderingStrategy::Hardware,
                };
            },
            MouseMode::Interface => {
                match self.emulation_active {
                    // UI mode with virtual mouse
                    true => {
                        glyph = match self.hide_emulated_cursor_over_ui {
                            false => self.interaction_state.into(),
                            true => CursorGlyph::None,
                        };
                        strategy = CursorRenderingStrategy::Emulated;
                    },
                    // UI mode with real mouse
                    false => {
                        glyph = self.interaction_state.into();
                        strategy = CursorRenderingStrategy::Hardware;
                    },
                };
            },

        }
        return VirtualMouseShellState {
            cursor_glyph: glyph,
            position: self.fused_state.position,
            cursor_rendering_strategy: strategy,
            mouse_mode: self.current_mode,
        };
    }

    pub fn left_press_state(&self) -> MousePressState {
        return self.left_press_state;
    }

    pub fn right_press_state(&self) -> MousePressState {
        return self.right_press_state;
    }

    pub fn left_is_pressed(&self) -> bool {
        return self.left_press_state.into();
    }

    pub fn right_is_pressed(&self) -> bool {
        return self.right_press_state.into();
    }

    pub fn delta(&self) -> Delta {
        return self.fused_state.delta;
    }

    pub fn position(&self) -> Position {
        return self.fused_state.position;
    }

    pub fn set_interaction_state(&mut self,interaction_state: InteractionState) {
        self.interaction_state = interaction_state;
    }

    pub fn set_mouse_mode(&mut self,new_mode: MouseMode) {
        self.current_mode = new_mode;
    }

    pub fn get_mouse_mode(&self) -> MouseMode {
        return self.current_mode;
    }

    pub fn set_camera_center_crosshair(&mut self,show_crosshair: bool) {
        self.hide_camera_center_crosshair = !show_crosshair;
    }

    pub fn set_emulated_cursor_over_ui(&mut self,show_pointer: bool) {
        self.hide_emulated_cursor_over_ui = !show_pointer;
    }
}
