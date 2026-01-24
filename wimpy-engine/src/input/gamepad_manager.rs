use gilrs::{
    Axis,
    Button,
    EventType,
    Gamepad,
    GamepadId,
    Gilrs,
    Error::NotImplemented,
    ev::state::{
        AxisData,
        ButtonData
    }
};

use crate::input::{
    Direction, ImpulseSet, ImpulseSetDescription, ImpulseState, InterpretiveAxes, InterpretiveAxis
};

pub struct GamepadManager {
    gilrs: Gilrs,
    active_gamepad_id: Option<GamepadId>,
    gamepad_output: GamepadOutput
}

struct GamepadOutput {
    interpretive_axes: InterpretiveAxes,
    impulse_set: ImpulseSet
}

impl Default for GamepadOutput {
    fn default() -> Self {
        Self {
            interpretive_axes: Default::default(),
            impulse_set: Default::default()
        }
    }
}

pub enum GamepadManagerError {
    UnsupportedGilrsPlatform,
    UnknownGilrsError
}

#[derive(PartialEq,Eq)]
pub enum UserActivity {
    None,
    Some
}

const THUMBSTICK_IMPULSE_THRESHOLD: f32 = 0.5;

impl GamepadManager {
    pub fn new() -> Result<Self,GamepadManagerError> {
        let gilrs = match Gilrs::new() {
            Ok(value) => value,
            Err(NotImplemented(_)) => {
                return Err(GamepadManagerError::UnsupportedGilrsPlatform);
            },
            Err(_) => {
                return Err(GamepadManagerError::UnknownGilrsError);
            },
        };
        return Ok(Self {
            gilrs,
            active_gamepad_id: None,
            gamepad_output: Default::default()
        });
    }

    fn update_active_gamepad(&mut self) {
        if let Some(id) = self.active_gamepad_id && self.gilrs.gamepad(id).is_connected() {
            return;
        }
        let new_id = None;
        for(id,gamepad) in self.gilrs.gamepads() {
            if gamepad.is_connected() {
                self.active_gamepad_id = Some(id);
                break;
            }
        }
        match (new_id,self.active_gamepad_id) {
            (None, Some(new_id)) => {
                let new = self.gilrs.gamepad(new_id);
                log::info!("Active gamepad set to '{}' (UUID: {:?}).",new.name(),new.uuid());
            },
            (Some(old_id), None) => {
                let old = self.gilrs.gamepad(old_id);
                log::info!("Gamepad '{}' (UUID: {:?}) is no longer active.",old.name(),old.uuid())
            },
            (Some(old_id),Some(new_id)) => {
                let old = self.gilrs.gamepad(old_id);
                let new = self.gilrs.gamepad(new_id);
                log::info!("Active gamepad '{}' (UUID: {:?}) replaced with '{}' (UUID: {:?}).",old.name(),old.uuid(),new.name(),new.uuid());
            },
            _ => {}
        };
        self.active_gamepad_id = new_id;
    }

    pub fn update(&mut self) -> UserActivity {

        let mut user_activity = UserActivity::None;

        while let Some(event) = self.gilrs.next_event() {
            let gamepad = self.gilrs.gamepad(event.id);
            match event.event {
                EventType::Connected => {
                    log::info!("Gamepad '{}' connected (UUID: {:?})",gamepad.name(),gamepad.uuid());
                },
                EventType::Disconnected => {
                    log::info!("Gamepad '{}' disconnected (UUID: {:?})",gamepad.name(),gamepad.uuid());
                }
                EventType::ButtonChanged(_,_,_) | EventType::AxisChanged(_,_,_) => {
                    if let Some(id) = self.active_gamepad_id && id == event.id {
                        user_activity = UserActivity::Some;
                    }
                },
                _ => {},
            }
        }

        self.update_active_gamepad();

        self.gamepad_output = match self.active_gamepad_id {
            Some(id) => {
                let gamepad = self.gilrs.gamepad(id);
                let interpretive_axes = get_interpretive_axes(&gamepad);
                let impulse_set = get_impulse_set(&gamepad,&interpretive_axes);
                GamepadOutput { interpretive_axes, impulse_set, }
            },
            None => Default::default(),
        };

        return user_activity;
    }

    pub fn get_axes(&self) -> &InterpretiveAxes {
        return &self.gamepad_output.interpretive_axes;
    }

    pub fn get_impulse_set(&self) -> &ImpulseSet {
        return &self.gamepad_output.impulse_set;
    }
}

pub fn get_impulse_set(gamepad: &Gamepad<'_>,axes: &InterpretiveAxes) -> ImpulseSet {

    let threshold = THUMBSTICK_IMPULSE_THRESHOLD;

    ImpulseSet::new(ImpulseSetDescription {
        up:    axes.infer_impulse(Direction::Up,threshold),
        down:  axes.infer_impulse(Direction::Down,threshold),
        left:  axes.infer_impulse(Direction::Left,threshold),
        right: axes.infer_impulse(Direction::Right,threshold),

        confirm: is_pressed(gamepad,Button::South),
        cancel:  is_pressed(gamepad,Button::East),

        focus_left:  is_pressed(gamepad,Button::LeftTrigger),
        focus_right: is_pressed(gamepad,Button::RightTrigger),

        view: is_pressed(gamepad,Button::Select),
        menu: is_pressed(gamepad,Button::Start),
    })
}

fn get_interpretive_axes(gamepad: &Gamepad<'_>) -> InterpretiveAxes {
    let axes = get_dpad_interpretive_axes(&gamepad);

    match axes.is_zero() {
        false => axes,
        true => InterpretiveAxes {
            x: translate_gamepad_axis(gamepad.axis_data(Axis::LeftStickX)),
            y: translate_gamepad_axis(gamepad.axis_data(Axis::LeftStickY)),
        },
    }
}

fn translate_gamepad_axis(axis_data: Option<&AxisData>) -> InterpretiveAxis {
    match axis_data {
        Some(axis_data) => InterpretiveAxis::from_f32(axis_data.value()),
        None => Default::default(),
    }
}

fn is_pressed(gamepad: &Gamepad<'_>,button: gilrs::ev::Button) -> ImpulseState {
    match gamepad.button_data(button) {
        Some(value) => ImpulseState::from_bool(value.is_pressed()),
        None => ImpulseState::Released,
    }
}

fn get_dpad_interpretive_axes(gamepad: &Gamepad<'_>) -> InterpretiveAxes {
    return InterpretiveAxes {
        x: InterpretiveAxis::from_impulse_state(
            is_pressed(gamepad,Button::DPadLeft),
            is_pressed(gamepad,Button::DPadRight),
        ),
        y: InterpretiveAxis::from_impulse_state(
            is_pressed(gamepad,Button::DPadUp),
            is_pressed(gamepad,Button::DPadDown),
        ),
    }
}
