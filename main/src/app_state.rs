#![allow(dead_code)]

use wimpy::{
    pipeline_management::Pipeline
};

use winit::keyboard::KeyCode;
use crate::graphics::Graphics;

pub struct UpdateResult {
    pub operation: AppOperation,
    pub new_state: Option<AppStateGenerator>
}

impl Default for UpdateResult {
    fn default() -> Self {
        return UpdateResult {
            operation: AppOperation::Continue,
            new_state: None 
        }
    }
}

pub enum AppOperation {
    Continue,
    Terminate,
    Transition
}

pub struct MousePoint {
    x: i32,
    y: i32
}

pub enum InputEvent {
    WindowSize(MousePoint), /* Sent after state load and resize (1) */
    MouseMove(MousePoint), /* Sent after state load and before mouse press and release (2) */

    MousePress(MousePoint), /* Not sent after load if pressed through transition.  */
    MouseRelease(MousePoint), /* Not sent unless mouse press started on the active state. */

    KeyPress(KeyCode), /* Sent after load if keys pressed through transition. */
    KeyRelease(KeyCode), /* Not sent to an unloading state */

    MouseMoveRaw((f64,f64))

    /* could also making the loading implementation parameterized */
}

pub trait AppStateHandler {
    fn unload(&mut self,graphics: &Graphics,pipeline: &mut Pipeline);

    fn update(&mut self) -> UpdateResult;
    fn input(&mut self,event: InputEvent);

    fn render(&self,graphics: &Graphics,pipeline: &mut Pipeline);
}

pub type AppState = Box<dyn AppStateHandler>;
pub type AppStateGenerator = fn(&Graphics,pipeline: &mut Pipeline) -> AppState;
