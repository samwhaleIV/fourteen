use winit::keyboard::KeyCode;
use super::virtual_device::VirtualDevice;
use crate::graphics::GraphicsContext;

pub struct UpdateResult {
    operation: AppStateOperation,
    state_generator: Option<AppStateGenerator>
}

impl Default for UpdateResult {
    fn default() -> Self {
        return UpdateResult {
            operation: AppStateOperation::Continue,
            state_generator: None 
        }
    }
}

#[allow(unused)]
impl UpdateResult {
    pub fn get_operation(&self) -> AppStateOperation {
        return self.operation;
    }

    pub fn get_state_generator(&self) -> Option<AppStateGenerator> {
        return self.state_generator;
    }

    pub fn transition(generator: AppStateGenerator) -> Self {
        return Self {
            operation: AppStateOperation::Transition,
            state_generator: Some(generator),
        }
    }
    pub fn terminate() -> Self {
        return Self {
            operation: AppStateOperation::Terminate,
            state_generator: None
        }
    }
}

#[derive(Clone,Copy)]
pub enum AppStateOperation {
    Continue,
    Terminate,
    Transition
}

#[allow(unused)]
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

pub trait AppStateInterface {
    fn unload(&mut self,virtual_device: &VirtualDevice,graphics_context: &mut GraphicsContext);

    fn update(&mut self) -> UpdateResult;
    fn input(&mut self,input_event: InputEvent);

    fn render(&self,virtual_device: &VirtualDevice,graphics_context: &mut GraphicsContext);
}

pub type AppState = Box<dyn AppStateInterface>;
pub type AppStateGenerator = fn(&VirtualDevice,&mut GraphicsContext) -> AppState;
