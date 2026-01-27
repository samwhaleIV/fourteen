use gilrs::*;
use wimpy_engine::input::GamepadInput;

pub struct GilrsManager {
    gilrs: Option<Gilrs>,
    active_gamepad: Option<GamepadId>,
}

impl Default for GilrsManager {
    fn default() -> Self {
        let gilrs = match Gilrs::new() {
            Ok(value) => Some(value),
            Err(error) => {
                log::warn!("Gilrs error: {:?}",error);
                None
            },
        };
        return Self {
            gilrs,
            active_gamepad: None,
        }
    }
}

impl GilrsManager {


    pub fn update(&mut self) -> GamepadInput {

        let Some(gilrs) = &mut self.gilrs else {
            return Default::default(); // gilrs never initialized
        };

        while let Some(event) = gilrs.next_event() {
            log::info!("{:?}",event.event);
            match event.event {
                EventType::Connected => {
                    if self.active_gamepad.is_none() {
                        self.active_gamepad = Some(event.id);
                    }
                },
                EventType::Disconnected => {
                    if let Some(id) = self.active_gamepad && id == event.id {
                        self.active_gamepad = None;
                    }
                },
                _ => {},
            }
        }

        let Some(id) = self.active_gamepad else {
            return Default::default(); // we do not have an active, connected gamepad, according to gilrs
        };

        let gamepad = gilrs.gamepad(id);

        let Some(axis_data) = gamepad.axis_data(Axis::LeftStickY) else {
            return Default::default();
        };

        log::info!("GILRS: Left Stick Y: {}",axis_data.value());

        return Default::default();
    }

}
