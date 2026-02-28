use super::prelude::*;
use bitflags::bitflags;

#[derive(Default)]
pub struct GamepadCache {
    input: GamepadInput,
    output: GamepadOutput
}

const JOYSTICK_IMPULSE_THRESHOLD: f32 = 0.5; //TODO: have an ascending and descending value in case the change isn't monotonic
const TRIGGER_IS_PRESSED_THREHOLD: f32 = 0.5; //TODO: have an ascending and descending value in case the change isn't monotonic

pub const AXIS_DEADZONE: f32 = 0.15;

bitflags! {
    #[derive(Debug,PartialEq,Eq,Copy,Clone)]
    pub struct GamepadButtons: u16 {
        const DPAD_UP = 1 << 0;
        const DPAD_DOWN = 1 << 1;
        const DPAD_LEFT = 1 << 2;
        const DPAD_RIGHT = 1 << 3;

        /// Also known as "view" button.
        const SELECT = 1 << 4;
        /// Also known as "start" button.
        const START = 1 << 5;

        const GUIDE = 1 << 6;

        const A = 1 << 7;
        const B = 1 << 8;
        const X = 1 << 9;
        const Y = 1 << 10;

        const LEFT_BUMPER = 1 << 11;
        const RIGHT_BUMPER = 1 << 12;

        const LEFT_STICK = 1 << 13;
        const RIGHT_STICK = 1 << 14;
    }
}

pub struct GamepadButtonSet {
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,

    pub select: bool,
    pub start: bool,
    pub guide: bool,

    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,

    pub left_bumper: bool,
    pub right_bumper: bool,

    pub left_stick: bool,
    pub right_stick: bool,
}

impl GamepadButtons {
    pub fn from_set(
        set: GamepadButtonSet
    ) -> Self {
        let mut buttons = Self::default();

        buttons.set(Self::DPAD_UP,set.dpad_up);
        buttons.set(Self::DPAD_DOWN,set.dpad_down);
        buttons.set(Self::DPAD_LEFT,set.dpad_left);
        buttons.set(Self::DPAD_RIGHT,set.dpad_right);

        buttons.set(Self::SELECT,set.select);
        buttons.set(Self::START,set.start);
        buttons.set(Self::GUIDE,set.guide);

        buttons.set(Self::A,set.a);
        buttons.set(Self::B,set.b);
        buttons.set(Self::X,set.x);
        buttons.set(Self::Y,set.y);

        buttons.set(Self::LEFT_BUMPER,set.left_bumper);
        buttons.set(Self::RIGHT_BUMPER,set.right_bumper);

        buttons.set(Self::LEFT_STICK,set.left_stick);
        buttons.set(Self::RIGHT_STICK,set.right_stick);

        buttons
    }
}

#[derive(Default,Copy,Clone)]
pub struct InterpetiveTrigger {
    value: f32,
    is_pressed: bool,
}

impl InterpetiveTrigger {
    fn create(value: f32) -> Self {
        return Self {
            value,
            is_pressed: value >= TRIGGER_IS_PRESSED_THREHOLD
        }
    }
    pub fn is_pressed(&self) -> bool {
        return self.is_pressed;
    }
    pub fn value(&self) -> f32 {
        return self.value;
    }
}

#[derive(Default)]
struct GamepadOutput {
    left_interpretive_axes: InterpretiveAxes,
    right_interpretive_axes: InterpretiveAxes,
    left_trigger: InterpetiveTrigger,
    right_trigger: InterpetiveTrigger,
    impulse_set: ImpulseSet,
}

impl Default for GamepadButtons {
    fn default() -> Self {
        Self::empty()
    }
}

impl GamepadButtons {
    fn impulse_state(&self,button: Self) -> ImpulseState {
        return ImpulseState::from_bool(self.contains(button));
    }
    fn bool(&self,button: Self) -> bool {
        return self.contains(button);
    }
}

#[derive(Debug,Default,PartialEq,Clone,Copy)]
pub struct GamepadJoystick {
    pub x: f32,
    pub y: f32,
}

impl GamepadJoystick {
    fn get_interpretive_axes(&self) -> InterpretiveAxes {
        InterpretiveAxes {
            x: InterpretiveAxis::from_f32_with_deadzone(self.x),
            y: InterpretiveAxis::from_f32_with_deadzone(self.y),
        }
    }
}

#[derive(Debug,Default,PartialEq)]
pub struct GamepadInput {
    pub buttons: GamepadButtons,

    pub left_stick: GamepadJoystick,
    pub right_stick: GamepadJoystick,

    pub left_trigger: f32,
    pub right_trigger: f32,
}

impl GamepadInput {
    fn get_left_interpretive_axes(&self) -> InterpretiveAxes {
        let axes = self.get_dpad_interpretive_axes();

        match axes.is_zero() {
            false => axes,
            true => self.left_stick.get_interpretive_axes(),
        }
    }

    fn get_right_interpretive_axes(&self) -> InterpretiveAxes {
        self.right_stick.get_interpretive_axes()
    }

    fn get_dpad_interpretive_axes(&self) -> InterpretiveAxes {
        InterpretiveAxes {
            x: InterpretiveAxis::from_bool(
                self.buttons.bool(GamepadButtons::DPAD_LEFT),
                self.buttons.bool(GamepadButtons::DPAD_RIGHT),
            ),
            y: InterpretiveAxis::from_bool(
                self.buttons.bool(GamepadButtons::DPAD_UP),
                self.buttons.bool(GamepadButtons::DPAD_DOWN),
            ),
        }
    }

    fn get_impulse_set(&self,axes: &InterpretiveAxes) -> ImpulseSet {

        let threshold = JOYSTICK_IMPULSE_THRESHOLD;

        ImpulseSet::new(ImpulseSetDescription {
            up:    axes.infer_impulse(Direction::Up,threshold),
            down:  axes.infer_impulse(Direction::Down,threshold),
            left:  axes.infer_impulse(Direction::Left,threshold),
            right: axes.infer_impulse(Direction::Right,threshold),

            confirm: self.buttons.impulse_state(GamepadButtons::A),
            cancel:  self.buttons.impulse_state(GamepadButtons::B),

            focus_left:  self.buttons.impulse_state(GamepadButtons::LEFT_BUMPER),
            focus_right: self.buttons.impulse_state(GamepadButtons::RIGHT_BUMPER),

            view: self.buttons.impulse_state(GamepadButtons::SELECT),
            menu: self.buttons.impulse_state(GamepadButtons::START),

            z_up: ImpulseState::Released,
            z_down: ImpulseState::Released,
        })
    }

    fn get_output(&self) -> GamepadOutput {
        let left_interpretive_axes = self.get_left_interpretive_axes();
        let right_interpretive_axes = self.get_right_interpretive_axes();

        let impulse_set = self.get_impulse_set(&left_interpretive_axes);

        let left_trigger = InterpetiveTrigger::create(self.left_trigger);
        let right_trigger = InterpetiveTrigger::create(self.right_trigger);

        return GamepadOutput {
            left_interpretive_axes,
            right_interpretive_axes,
            left_trigger,
            right_trigger,
            impulse_set
        };
    }
}

impl GamepadCache {
    pub fn update(&mut self,gamepad_input: GamepadInput) -> UserActivity {
        let user_activity = match &self.input.buttons != &gamepad_input.buttons {
            true => UserActivity::Some,
            false => UserActivity::None,
        };

        self.input = gamepad_input;
        self.output = self.input.get_output();

        return user_activity;
    }

    pub fn left_axes(&self) -> InterpretiveAxes {
        return self.output.left_interpretive_axes;
    }

    pub fn right_axes(&self) -> InterpretiveAxes {
        return self.output.right_interpretive_axes;
    }

    pub fn impulse_set(&self) -> &ImpulseSet {
        return &self.output.impulse_set;
    }

    pub fn left_trigger(&self) -> InterpetiveTrigger {
        return self.output.left_trigger;
    }

    pub fn right_trigger(&self) -> InterpetiveTrigger {
        return self.output.right_trigger;
    }
}
