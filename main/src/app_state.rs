#![allow(dead_code)]

use wimpy::graphics::Pipeline;

use winit::keyboard::KeyCode;
use crate::graphics_binder::GraphicsBinder;

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

pub enum InputEvent {
    WindowSize((f32,f32)), /* Sent after state load and resize (1) */
    MouseMove((f32,f32)), /* Sent after state load and before mouse press and release (2) */

    MousePress((f32,f32)), /* Not sent after load if pressed through transition.  */
    MouseRelease((f32,f32)), /* Not sent unless mouse press started on the active state. */

    KeyPress(KeyCode), /* Sent after load if keys pressed through transition. */
    KeyRelease(KeyCode), /* Not sent to an unloading state */

    MouseMoveDelta((f32,f32))
    /* could also making the loading implementation parameterized */
}

pub trait AppStateHandler {
    fn unload(&mut self,graphics: &GraphicsBinder,pipeline: &mut Pipeline);

    fn update(&mut self) -> UpdateResult;
    fn input(&mut self,event: InputEvent);

    fn render(&self,graphics: &GraphicsBinder,pipeline: &mut Pipeline);
}

pub type AppState = Box<dyn AppStateHandler>;
pub type AppStateGenerator = fn(&GraphicsBinder,pipeline: &mut Pipeline) -> AppState;
