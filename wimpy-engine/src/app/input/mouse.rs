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
    pub right_pressed: bool
}

#[derive(Clone,Copy)]
pub enum MouseModeSwitchCommand {
    InterfaceToCamera,
    CameraToInterface,
}

impl MouseInput {
    fn from_gamepad(gamepad: &GamepadCache,position: Position,emulation_bounds: WimpyArea,delta_seconds: f32) -> Self {
        let max_pixels = JOYSTICK_CURSOR_PIXELS_PER_SECOND * delta_seconds;
        let delta = Delta {
            x: gamepad.right_axes().get_x_f32() * max_pixels,
            y: gamepad.right_axes().get_y_f32() * max_pixels
        };
        return Self {
            position: position.add_delta(&delta).clip(emulation_bounds),
            delta,
            left_pressed: gamepad.left_trigger().is_pressed(),
            right_pressed: gamepad.right_trigger().is_pressed(),
        }
    }
    fn delta_changed(&self,other: &Self) -> bool {
        self.delta != other.delta
    }
    fn fuse_buttons(&self,other: &Self) -> Self {
        return Self {
            position: self.position,
            delta: self.delta,
            left_pressed: self.left_pressed || other.left_pressed,
            right_pressed: self.right_pressed || other.right_pressed,
        }
    }
}

#[derive(Default)]
pub struct VirtualMouse {

    state: MouseInput,

    last_mouse_state: MouseInput,
    last_gamepad_state: MouseInput,

    left_press_state: MousePressState,
    right_press_state: MousePressState,

    virtual_mouse_mode: MouseMode,
    mode_switch_command: Option<MouseModeSwitchCommand>,

    interaction_state: InteractionState,
    emulation_active: bool,

    hide_emulated_cursor_over_ui: bool,
    hide_camera_center_crosshair: bool,

    center_screen: (f32,f32)
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

#[derive(Default,Copy,Clone,PartialEq,Eq)]
pub enum MouseMode {
    #[default]
    Interface,
    Camera
}

#[derive(Default,Copy,Clone,PartialEq,Eq)]
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
    pub cursor_x: f32,
    pub cursor_y: f32,
    pub cursor_rendering_strategy: CursorRenderingStrategy,
    pub mode_switch_command: Option<MouseModeSwitchCommand>
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
        mouse_state: MouseInput,
        gamepad: &GamepadCache,
        delta_seconds: f32,
        emulation_bounds: WimpyArea,
    ) -> VirtualMouseShellState {
        self.center_screen = emulation_bounds.center();
        let mode_switch_command = self.mode_switch_command;
        self.mode_switch_command = None;

        let gamepad_state = MouseInput::from_gamepad(
            gamepad,
            self.state.position,
            emulation_bounds,
            delta_seconds
        );

        let gamepad_state_changed = gamepad_state.delta_changed(&self.last_gamepad_state);
        let mouse_state_changed = mouse_state.delta_changed(&self.last_mouse_state);

        self.last_gamepad_state = gamepad_state;
        self.last_mouse_state = mouse_state;

        match (gamepad_state_changed,mouse_state_changed) {
            (false, true) | (true, true) => {
                self.state = mouse_state.fuse_buttons(&gamepad_state);
                self.emulation_active = false;
            },
            (true, false) => {
                self.state = gamepad_state.fuse_buttons(&mouse_state);
                self.emulation_active = true;
            },
            (false, false) => match self.emulation_active {
                true => {
                    self.state = gamepad_state.fuse_buttons(&mouse_state);
                },
                false => {
                    self.state = mouse_state.fuse_buttons(&gamepad_state);
                },
            },
        }

        self.left_press_state = get_delta_press_state(
            self.left_press_state,
            self.state.left_pressed
        );

        self.right_press_state = get_delta_press_state(
            self.right_press_state,
            self.state.right_pressed
        );

        return self.get_cursor_shell_state(mode_switch_command);
    }

    pub fn get_left_press_state(&self) -> MousePressState {
        return self.left_press_state;
    }

    pub fn get_right_press_state(&self) -> MousePressState {
        return self.right_press_state;
    }

    pub fn left_is_pressed(&self) -> bool {
        return self.left_press_state.into();
    }

    pub fn right_is_pressed(&self) -> bool {
        return self.right_press_state.into();
    }

    pub fn get_delta(&self) -> Delta {
        return self.state.delta;
    }

    pub fn get_position(&self) -> Position {
        return self.state.position;
    }

    pub fn set_emulated_position(&mut self,position: Position) {
        if self.emulation_active {
            self.state.delta = Default::default();
            self.state.position = position;
        }
    }

    pub fn set_interaction_state(&mut self,interaction_state: InteractionState) {
        self.interaction_state = interaction_state;
    }

    pub fn set_mouse_mode(&mut self,new_mode: MouseMode) {
        let old_mode = self.virtual_mouse_mode;
        self.virtual_mouse_mode = new_mode;
        match (old_mode,new_mode) {
            (MouseMode::Interface, MouseMode::Camera) => {
                self.mode_switch_command = Some(MouseModeSwitchCommand::InterfaceToCamera);
            },
            (MouseMode::Camera, MouseMode::Interface) => {
                self.mode_switch_command = Some(MouseModeSwitchCommand::CameraToInterface);
                self.set_emulated_position(Position {
                    x: self.center_screen.0,
                    y: self.center_screen.1
                });
            },
            _ => {}
        };
    }

    pub fn enable_center_crosshair_in_camera_mode(&mut self) {
        self.hide_camera_center_crosshair = false;
    }

    pub fn disable_center_crosshair_in_camera_mode(&mut self) {
        self.hide_camera_center_crosshair = true;
    }

    pub fn enable_emulated_crosshair_over_ui(&mut self) {
        self.hide_emulated_cursor_over_ui = false;
    }

    pub fn disable_emulated_crosshair_over_ui(&mut self) {
        self.hide_emulated_cursor_over_ui = true;
    }

    fn get_cursor_shell_state(&self,mode_switch_command: Option<MouseModeSwitchCommand>) -> VirtualMouseShellState {
        if self.virtual_mouse_mode == MouseMode::Camera {
            // Camera control mode with any mouse
            return VirtualMouseShellState {
                cursor_glyph: match self.hide_camera_center_crosshair {
                    false => CursorGlyph::CameraCrosshair,
                    true => CursorGlyph::None,
                },
                cursor_rendering_strategy: CursorRenderingStrategy::Emulated,
                mode_switch_command,
                cursor_x: self.state.position.x,
                cursor_y: self.state.position.y,
            };
        }
        match self.emulation_active {
            // UI mode with virtual mouse
            true => VirtualMouseShellState {
                cursor_glyph: match self.hide_emulated_cursor_over_ui {
                    false => self.interaction_state.into(),
                    true => CursorGlyph::None,
                },
                cursor_rendering_strategy: CursorRenderingStrategy::Emulated,
                mode_switch_command,
                cursor_x: self.state.position.x,
                cursor_y: self.state.position.y,
            },
            // UI mode with real mouse
            false => VirtualMouseShellState {
                cursor_glyph: self.interaction_state.into(),
                cursor_rendering_strategy: CursorRenderingStrategy::Hardware,
                mode_switch_command,
                cursor_x: self.state.position.x,
                cursor_y: self.state.position.y,
            },
        }
    }
}
