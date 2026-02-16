use crate::shared::WimpyArea;
use super::prelude::*;

const JOYSTICK_CURSOR_PIXELS_PER_SECOND: f32 = 1500.0;

#[derive(Default,Copy,Clone,PartialEq,Debug)]
pub struct MousePosition {
    pub x: f32,
    pub y: f32
}

impl MousePosition {
    fn add_delta(&mut self,delta: &MouseDelta) -> &mut Self {
        self.x += delta.x;
        self.y += delta.y;
        self
    }
    fn clip(&mut self,bounds: WimpyArea) -> &mut Self {
        let point = bounds.clip(self.x,self.y);
        self.x = point.0;
        self.y = point.1;
        self
    }
}

#[derive(Default,Copy,Clone,PartialEq,Debug)]
pub struct MouseDelta {
    pub x: f32,
    pub y: f32
}

#[derive(Default,PartialEq)]
pub struct MouseInput {
    pub position: MousePosition,
    pub delta: MouseDelta,
    pub left_pressed: bool,
    pub right_pressed: bool
}

#[derive(Clone,Copy)]
pub enum MouseModeSwitchCommand {
    InterfaceToCamera,
    CameraToInterface,
}

pub struct VirtualMouse {
    left_click_state: MousePressState,
    right_click_state: MousePressState,

    position: MousePosition,
    delta: MouseDelta,

    virtual_mouse_mode: MouseMode,
    mode_switch_command: Option<MouseModeSwitchCommand>,

    interaction_state: InteractionState,
    emulation_active: bool,

    show_emulated_cursor_over_ui: bool,
    show_camera_center_crosshair: bool,

    center_screen: (f32,f32)
}

impl Default for VirtualMouse {
    fn default() -> Self {
        Self {
            left_click_state: Default::default(),
            right_click_state: Default::default(),
            position: Default::default(),
            delta: Default::default(),
            virtual_mouse_mode: Default::default(),
            mode_switch_command: Default::default(),
            interaction_state: Default::default(),
            emulation_active: Default::default(),
            show_emulated_cursor_over_ui: true,
            show_camera_center_crosshair: true,
            center_screen: Default::default(),
        }
    }
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
    fn update_with_mouse(&mut self,mouse_input: &MouseInput) {
        self.delta = mouse_input.delta;
        self.position = mouse_input.position;
        self.left_click_state = get_delta_press_state(
            self.left_click_state,
            mouse_input.left_pressed
        );
        self.right_click_state = get_delta_press_state(
            self.right_click_state,
            mouse_input.right_pressed
        );
        self.emulation_active = false;
    }

    fn update_with_gamepad(&mut self,gamepad: &GamepadCache,emulation_bounds: WimpyArea,delta_seconds: f32) {
        let max_pixels = JOYSTICK_CURSOR_PIXELS_PER_SECOND * delta_seconds;
        self.delta = MouseDelta {
            x: gamepad.right_axes().get_x_f32() * max_pixels,
            y: gamepad.right_axes().get_y_f32() * max_pixels
        };
        self.position.add_delta(&self.delta).clip(emulation_bounds);
        self.left_click_state = get_delta_press_state(
            self.left_click_state,
            gamepad.left_trigger().is_pressed()
        );
        self.right_click_state = get_delta_press_state(
            self.right_click_state,
            gamepad.right_trigger().is_pressed()
        );
        self.emulation_active = true;
    }

    pub(super) fn update(
        &mut self,
        mouse: &MouseInput,
        gamepad: &GamepadCache,
        recent_input_type: InputType,
        delta_seconds: f32,
        emulation_bounds: WimpyArea,
    ) -> VirtualMouseShellState {
        self.center_screen = emulation_bounds.center();
        let mode_switch_command = self.mode_switch_command;
        self.mode_switch_command = None;
        match recent_input_type {
            InputType::Unknown | InputType::KeyboardAndMouse => self.update_with_mouse(mouse),
            InputType::Gamepad => self.update_with_gamepad(gamepad,emulation_bounds,delta_seconds),
        };
        return self.get_cursor_shell_state(mode_switch_command);
    }

    pub fn get_left_press_state(&self) -> MousePressState {
        return self.left_click_state;
    }

    pub fn get_right_press_state(&self) -> MousePressState {
        return self.right_click_state;
    }

    pub fn left_is_pressed(&self) -> bool {
        return self.left_click_state.into();
    }

    pub fn right_is_pressed(&self) -> bool {
        return self.right_click_state.into();
    }

    pub fn get_delta(&self) -> MouseDelta {
        return self.delta;
    }

    pub fn get_position(&self) -> MousePosition {
        return self.position;
    }

    pub fn set_emulated_position(&mut self,position: MousePosition) {
        if self.emulation_active {
            self.delta = Default::default();
            self.position = position;
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
                self.set_emulated_position(MousePosition {
                    x: self.center_screen.0,
                    y: self.center_screen.1
                });
            },
            _ => {}
        };
    }

    pub fn enable_center_crosshair_in_camera_mode(&mut self) {
        self.show_camera_center_crosshair = true;
    }

    pub fn disable_center_crosshair_in_camera_mode(&mut self) {
        self.show_camera_center_crosshair = false;
    }

    pub fn enable_emulated_crosshair_over_ui(&mut self) {
        self.show_emulated_cursor_over_ui = true;
    }

    pub fn disable_emulated_crosshair_over_ui(&mut self) {
        self.show_emulated_cursor_over_ui = false;
    }

    fn get_cursor_shell_state(&self,mode_switch_command: Option<MouseModeSwitchCommand>) -> VirtualMouseShellState {
        if self.virtual_mouse_mode == MouseMode::Camera {
            // Camera control mode with any mouse
            return VirtualMouseShellState {
                cursor_glyph: match self.show_camera_center_crosshair {
                    true => CursorGlyph::CameraCrosshair,
                    false => CursorGlyph::None,
                },
                cursor_rendering_strategy: CursorRenderingStrategy::Emulated,
                mode_switch_command,
                cursor_x: self.position.x,
                cursor_y: self.position.y
            };
        }
        match self.emulation_active {
            // UI mode with virtual mouse
            true => VirtualMouseShellState {
                cursor_glyph: match self.show_emulated_cursor_over_ui {
                    true => self.interaction_state.into(),
                    false => CursorGlyph::None,
                },
                cursor_rendering_strategy: CursorRenderingStrategy::Emulated,
                mode_switch_command,
                cursor_x: self.position.x,
                cursor_y: self.position.y
            },
            // UI mode with real mouse
            false => VirtualMouseShellState {
                cursor_glyph: self.interaction_state.into(),
                cursor_rendering_strategy: CursorRenderingStrategy::Hardware,
                mode_switch_command,
                cursor_x: self.position.x,
                cursor_y: self.position.y
            },
        }
    }
}
