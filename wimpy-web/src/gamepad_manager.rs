use wasm_bindgen::prelude::*;
use web_sys::js_sys::Float32Array;
use wimpy_engine::app::input::*;

const BUTTON_PRESS_THRESHOLD: f32 = 0.05;

#[wasm_bindgen(module = "/html/gamepad-manager.js")]
extern "C" {
    pub type GamepadManager;

    #[wasm_bindgen(constructor)]
    pub fn new() -> GamepadManager;

    #[wasm_bindgen(method,getter)]
    pub fn buffer(this: &GamepadManager) -> Float32Array;

    #[wasm_bindgen(method)]
    pub fn update(this: &GamepadManager);
}

fn to_bool(value: f32) -> bool {
    value > BUTTON_PRESS_THRESHOLD
}

fn axis_clamp(value: f32) -> f32 {
    value.min(1.0).max(-1.0)
}

fn trigger_clamp(value: f32) -> f32 {
    value.min(1.0).max(0.0)
}

pub fn create_gamepad_state(src: Float32Array) -> GamepadInput {
    GamepadInput {
        buttons: GamepadButtons::from_set(GamepadButtonSet {
            dpad_up:      to_bool(src.get_index(0)),
            dpad_down:    to_bool(src.get_index(1)),
            dpad_left:    to_bool(src.get_index(2)),
            dpad_right:   to_bool(src.get_index(3)),

            select:       to_bool(src.get_index(4)),
            start:        to_bool(src.get_index(5)),
            guide:        to_bool(src.get_index(6)),

            a:            to_bool(src.get_index(7)),
            b:            to_bool(src.get_index(8)),
            x:            to_bool(src.get_index(9)),
            y:            to_bool(src.get_index(10)),

            left_bumper:  to_bool(src.get_index(11)),
            right_bumper: to_bool(src.get_index(12)),

            left_stick:   to_bool(src.get_index(13)),
            right_stick:  to_bool(src.get_index(14)),
        }),
        left_stick: GamepadJoystick {
            x: axis_clamp(src.get_index(15)),
            y: axis_clamp(src.get_index(16)),
        },
        right_stick: GamepadJoystick {
            x: axis_clamp(src.get_index(17)),
            y: axis_clamp(src.get_index(18)),
        },
        left_trigger: trigger_clamp(src.get_index(19)),
        right_trigger: trigger_clamp(src.get_index(20)),
    }
}
