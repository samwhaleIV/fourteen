use winit::keyboard::KeyCode;
use super::virtual_device::VirtualDevice;
use crate::graphics::GraphicsContext;

pub struct UpdateResult<TSharedState> {
    operation: AppStateOperation,
    state_generator: Option<AppStateGenerator<TSharedState>>
}

impl<TSharedState> Default for UpdateResult<TSharedState> {
    fn default() -> Self {
        return UpdateResult {
            operation: AppStateOperation::Continue,
            state_generator: None 
        }
    }
}

#[allow(unused)]
impl<TSharedState> UpdateResult<TSharedState> {
    pub fn get_operation(&self) -> AppStateOperation {
        return self.operation;
    }

    pub fn get_state_generator(&self) -> Option<AppStateGenerator<TSharedState>> {
        return self.state_generator;
    }

    pub fn transition(generator: AppStateGenerator<TSharedState>) -> Self {
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

pub trait AppStateInterface<TSharedState> {

    fn input(&mut self,input_event: InputEvent);

    fn unload(&mut self,virtual_device: &VirtualDevice,graphics_context: &mut GraphicsContext);
    fn render(&mut self,virtual_device: &VirtualDevice,graphics_context: &mut GraphicsContext);
    fn update(&mut self,virtual_device: &VirtualDevice,graphics_context: &mut GraphicsContext) -> UpdateResult<TSharedState>;

    fn insert_shared_state(&mut self,shared_state: Option<TSharedState>);
    fn remove_shared_state(&mut self) -> Option<TSharedState>;
}

pub type AppState<TSharedState> = Box<dyn AppStateInterface<TSharedState>>;
pub type AppStateGenerator<TSharedState> = fn(&VirtualDevice,&mut GraphicsContext) -> AppState<TSharedState>;
