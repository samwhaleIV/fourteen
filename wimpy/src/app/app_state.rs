use winit::keyboard::KeyCode;
use crate::{
    app::{
        AppStateGenerator,
        VirtualDevice
    },
    wgpu::GraphicsContext
};

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


