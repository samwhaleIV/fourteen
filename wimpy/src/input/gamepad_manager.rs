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
    ImpulseSet,
    InterpretiveAxes,
    InterpretiveAxis
};

pub struct GamepadManager {
    gilrs: Gilrs,
    active_gamepad_id: Option<GamepadId>,
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
        });
    }

    fn try_set_active_gamepad(&mut self) {
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
        self.set_active_gamepad(new_id);
    }

    fn set_active_gamepad(&mut self,new_gamepad: Option<GamepadId>) {
        match (new_gamepad,self.active_gamepad_id) {
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
        self.active_gamepad_id = new_gamepad;
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

        self.try_set_active_gamepad();

        return user_activity;
    }

    pub fn get_impulse_set(&self) -> ImpulseSet {
        todo!();
    }

    pub fn get_axes(&self) -> InterpretiveAxes {
        let Some(id) = self.active_gamepad_id else {
            return Default::default();
        };

        let gamepad = self.gilrs.gamepad(id);

        let axes = get_dpad_interpretive_axes(&gamepad);

        match axes.is_zero() {
            false => axes,
            true => InterpretiveAxes {
                x: translate_gamepad_axis(gamepad.axis_data(Axis::LeftStickX)),
                y: translate_gamepad_axis(gamepad.axis_data(Axis::LeftStickY)),
            },
        }
    }
}

fn translate_gamepad_axis(value: Option<&AxisData>) -> InterpretiveAxis {
    value.map_or(Default::default(),|v|InterpretiveAxis::from_f32(v.value()))
}

fn is_pressed(value: Option<&ButtonData>) -> bool {
    value.map_or(false,|button|button.is_pressed())
}

fn get_dpad_interpretive_axes(gamepad: &Gamepad<'_>) -> InterpretiveAxes {
    return InterpretiveAxes {
        x: InterpretiveAxis::from_bool(
            is_pressed(gamepad.button_data(Button::DPadLeft)),
            is_pressed(gamepad.button_data(Button::DPadRight))
        ),
        y: InterpretiveAxis::from_bool(
            is_pressed(gamepad.button_data(Button::DPadUp)),
            is_pressed(gamepad.button_data(Button::DPadDown))
        ),
    }
}
