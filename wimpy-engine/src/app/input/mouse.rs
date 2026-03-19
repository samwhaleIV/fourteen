use crate::{WimpyRect, WimpyVec};
use super::{gamepad::GamepadCache, InputDevice};

#[derive(Clone,Copy)]
pub enum MouseModeSwitchCommand {
    InterfaceToCamera,
    CameraToInterface,
}


#[derive(Default,PartialEq,Copy,Clone)]
pub struct Input {
    pub position:       WimpyVec,
    pub delta:          WimpyVec,
    pub left_pressed:   bool,
    pub right_pressed:  bool,
}

impl Input {
    fn from_gamepad(
        gamepad:            &GamepadCache,
        position:           WimpyVec,
        retain_position:    bool,
        emulation_bounds:   WimpyRect,
        delta_seconds:      f32
    ) -> Self {
        use super::constants::JOYSTICK_CURSOR_PIXELS_PER_SECOND as pixels_per_second;
        let max_pixels = pixels_per_second * delta_seconds;
        let delta = WimpyVec::from(gamepad.right_axes()) * max_pixels;
        let position = match retain_position {
            true => position,
            false => emulation_bounds.clip(position + delta),
        };
        Self {
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
        Self {
            position: self.position,
            delta: self.delta,
            left_pressed: self.left_pressed || other.left_pressed,
            right_pressed: self.right_pressed || other.right_pressed,
        }
    }
    fn has_delta_activity(&self) -> bool {
        self.delta.x != 0.0 ||
        self.delta.y != 0.0
    }
}

#[derive(Default)]
pub struct VirtualMouse {
    fused_state: Input,

    mouse_state: Input,
    gamepad_state: Input,

    left_press_state: PressState,
    right_press_state: PressState,

    current_mode: MouseMode,
    future_mode: Option<MouseMode>,

    interaction_state: InteractionState,
    emulation_active: bool,

    hide_camera_center_crosshair: bool,

    gamepad_position_init: bool,
    initialized: bool,
}

#[derive(Default,Copy,Clone,PartialEq,Eq)]
pub enum PressState {
    #[default]
    Released,
    JustPressed,
    Pressed,
    JustReleased,
}

impl From<PressState> for bool {
    fn from(value: PressState) -> Self {
        use PressState::*;
        match value {
            Released =>     false,
            JustPressed =>  true,
            Pressed =>      true,
            JustReleased => false,
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

impl From<InteractionState> for CursorGlyph {
    fn from(value: InteractionState) -> Self {
        use InteractionState::*;
        match value {
            Hidden =>           Self::None,
            Default =>          Self::Default,
            CanInteract =>      Self::CanInteract,
            IsInteracting =>    Self::IsInteracting,
        }
    }
}

pub struct ShellState {
    pub glyph:      CursorGlyph,
    pub position:   WimpyVec,
    /// Specifies which hardware input device is most likely to be active to guide the display of software versus hardware cursor
    pub device:     InputDevice,
    pub mode:       MouseMode,
    /// If the hardware cursor is enabled, this flag is set when the virtual mouse determines the cursor *should* be recentered
    /// 
    /// This is only active for one frame at a time, such as when dropping out of camera control mode
    pub recenter:   bool
}

fn get_delta_press_state(old_state: PressState,is_pressed: bool) -> PressState {
    use PressState::*;
    match (old_state,is_pressed) {
        (Released,      true) =>    JustPressed,
        (Released,      false) =>   Released,
        (JustPressed,   true) =>    Pressed,
        (JustPressed,   false) =>   JustReleased,
        (Pressed,       true) =>    Pressed,
        (Pressed,       false) =>   JustReleased,
        (JustReleased,  true) =>    JustPressed,
        (JustReleased,  false) =>   Released,
    }
}

impl VirtualMouse {
    pub(super) fn update(
        &mut self,
        input_device_hint:  InputDevice,
        new_mouse_state:    Input,
        gamepad:            &GamepadCache,
        delta_seconds:      f32,
        emulation_bounds:   WimpyRect,
        can_recenter:       bool
    ) -> ShellState {

        let mut zero_out_delta = false;

        self.gamepad_state = Input::from_gamepad(
            gamepad,
            self.gamepad_state.position,
            self.current_mode == MouseMode::Camera,
            emulation_bounds,
            delta_seconds
        );

        let mut should_reposition_hardware_cursor = false;

        let previous_mouse_state = self.mouse_state;
        self.mouse_state = new_mouse_state;

        if let Some(new_mode) = self.future_mode.take() && new_mode != self.current_mode {
            self.current_mode = new_mode;
            zero_out_delta = true;
            if self.current_mode == MouseMode::Interface {
                let center = emulation_bounds.center();
                self.gamepad_state.position = center;
                if can_recenter {
                    self.mouse_state.position = center;
                    should_reposition_hardware_cursor = true;
                }
            }
            use InputDevice::*;
            self.emulation_active = match input_device_hint {
                MouseAndKeyboard => false,
                Gamepad =>          true,
            };
        }

        let has_mouse_activity_right_now = self.mouse_state.position_differs(&previous_mouse_state);

        match (
            self.gamepad_state.has_delta_activity(),
            has_mouse_activity_right_now
        ) {
            /* Mouse control */
            (false, true) => {
                self.fused_state = self.mouse_state.create_fused(&self.gamepad_state);
                self.emulation_active = false;
            },
            /* Gamepad control */
            (true, false) => {
                self.fused_state = self.gamepad_state.create_fused(&self.mouse_state);
                self.emulation_active = true;
            },
            /* Reuse previous frame's priority. Short circuit for if both devices change in the same frame. (Or a synthetic zero-delta mouse event got generated) */
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

        match self.emulation_active {
            /* Gamepad active */
            true => {
                /* First position fix - start in center screen if the mouse was never activated */
                if !self.gamepad_position_init {
                    self.gamepad_position_init = true;
                    self.gamepad_state.position = emulation_bounds.center();
                    if can_recenter {
                        should_reposition_hardware_cursor = true;
                    }
                    /* If we don't update the output state, there will be a single frame error with the cursor displayed at the origin. */
                    self.fused_state.position = self.gamepad_state.position;
                } else if
                    can_recenter &&
                    self.current_mode == MouseMode::Interface
                {
                    should_reposition_hardware_cursor = true;
                }
            }

            /* Mouse active */
            false => {
                /* Update the gamepad state behind the scenes (it is inactive) */
                self.gamepad_state.position = self.fused_state.position;
                if
                    !self.gamepad_position_init &&
                    self.initialized &&
                    has_mouse_activity_right_now &&
                    self.current_mode == MouseMode::Interface
                {
                    self.gamepad_position_init = true;
                }
            },
        };

        self.initialized = true;

        if zero_out_delta {
            self.fused_state.delta = WimpyVec::ZERO;
        }

        return self.get_cursor_shell_state(should_reposition_hardware_cursor);
    }

    fn get_cursor_shell_state(&self,should_reposition_hardware_cursor: bool) -> ShellState {

        let glyph:  CursorGlyph;
        let device: InputDevice;

        match self.current_mode {
            MouseMode::Camera => {
                glyph = match self.hide_camera_center_crosshair {
                    false => CursorGlyph::CameraCrosshair,
                    true =>  CursorGlyph::None,
                };
                // Even though the cursor won't be visible, we prime the cursor rendering in anticipation of a mode swap
                device = match self.emulation_active {
                    true =>  InputDevice::Gamepad,
                    false => InputDevice::MouseAndKeyboard,
                };
            },
            MouseMode::Interface => {
                match self.emulation_active {
                    // UI mode with virtual mouse
                    true => {
                        glyph =  self.interaction_state.into();
                        device = InputDevice::Gamepad;
                    },
                    // UI mode with real mouse
                    false => {
                        glyph =  self.interaction_state.into();
                        device = InputDevice::MouseAndKeyboard;
                    },
                };
            },
        }

        ShellState {
            glyph,
            position: self.fused_state.position,
            device,
            mode: self.current_mode,
            recenter: should_reposition_hardware_cursor,
        }
    }

    pub fn left_press_state(&self) -> PressState {
        self.left_press_state
    }

    pub fn right_press_state(&self) -> PressState {
        self.right_press_state
    }

    pub fn left_is_pressed(&self) -> bool {
        self.left_press_state.into()
    }

    pub fn right_is_pressed(&self) -> bool {
        self.right_press_state.into()
    }

    /// Relative mouse motion between the last frame.
    /// 
    /// These deltas are not stable over position.
    /// The changes will not add up to the current position.
    /// 
    /// If the inetraction mode became the camera mode this frame, delta will be zero.
    pub fn delta(&self) -> WimpyVec {
        self.fused_state.delta
    }

    pub fn position(&self) -> WimpyVec {
        self.fused_state.position
    }

    /// A hint to the UI shell for which cursor glyph to use.
    /// 
    /// This does not change the behavior of input processing, it is purely cosmetic.
    /// Interaction states have to be conceptualized by the caller.
    pub fn set_interaction_state(&mut self,interaction_state: InteractionState) {
        self.interaction_state = interaction_state;
    }

    /// Schedules interaction mouse mode, i.e. UI mode to begin on the next frame.
    /// 
    /// Does not take immediate effect.
    /// 
    /// Will override another queued mouse mode.
    pub fn queue_interaction_mode(&mut self) {
        self.future_mode = Some(MouseMode::Interface);
    }

    /// Does not take immediate effect.
    /// 
    /// Schedules relative mouse mode, i.e. camera mode to begin on the next frame.
    /// 
    /// Will override another queued mouse mode.
    pub fn queue_camera_mode(&mut self) {
        self.future_mode = Some(MouseMode::Camera);
    }

    /// Since the real hardware mode can only change at the beginning of the next frame,
    /// this will only return the mode that is active for the duration of the current frame.
    pub fn get_active_mode(&self) -> MouseMode {
        self.current_mode
    }

    pub fn set_camera_crosshair_visibility(&mut self,visible: bool) {
        self.hide_camera_center_crosshair = !visible;
    }
}
