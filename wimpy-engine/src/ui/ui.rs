const MINIMUM_NODE_CAPACITY: usize = 32; /* Should be at least 1. */

use std::{
    time::{
        Duration,
        Instant
    }
};

use slotmap::{
    SlotMap,
    SparseSecondaryMap
};
use smallvec::SmallVec;

use crate::{
    shared::{
        Area,
        Color,
        Layout
    },
    ui::nodes::{
        Node,
        NodeContainer,
        NodeError
    }
};

pub enum MouseState {
    Released,
    Pressed,
}

enum AnimationEffect {
    FinalScale(f32),
    FadeIn,
    FadeOut,
}

pub enum InteractionMode {
    None,
    Block,
    Active
}

struct AnimationTiming {
    start: Instant,
    duration: Duration,
}

struct Animation {
    timing: AnimationTiming,
    effect: AnimationEffect,
}

struct LayoutAnimation {
    timing: AnimationTiming,
    future_layout: Layout
}

struct ElementAnimations {
    layout_animation: Option<LayoutAnimation>,
    focus_animation: Option<Animation>,
    hover_animation: Option<Animation>,
}

slotmap::new_key_type! {
    pub struct TextureReference;
}

pub trait LocateTexture<TTexture> {
    fn get(texture_reference: TextureReference) -> TTexture;
}

pub struct UINodeInput {
    pub layout: Layout,
    pub uv: Area,
    pub rotation: f32,
    pub color: Color,
    pub texture: Option<TextureReference>,
    pub interaction_mode: InteractionMode,

    pub clip_children: bool,
    pub is_root: bool
}

pub struct UINodeOutput {
    pub layout: Area,
    pub uv: Area,
    pub rotation: f32,
    pub color: Color,
    pub texture: Option<TextureReference>, 
}

impl Default for UINodeInput {
    fn default() -> Self {
        Self {
            layout: Default::default(),
            uv: Area::ONE,
            rotation: 0.0,
            color: Color::WHITE,
            texture: None,
            interaction_mode: InteractionMode::None,
            clip_children: false,
            is_root: false
        }
    }
}

impl Default for UINodeOutput {
    fn default() -> Self {
        Self {
            layout: Default::default(),
            uv: Area::ONE,
            rotation: 0.0,
            color: Color::WHITE,
            texture: None,
        }
    }
}

pub enum PageEvent {
    ClickStart(Node),
    ClickRelease(Node),
    FocusStart(Node),
    FocusEnd(Node),
    FocusLost
}

pub struct Page<TTexture> {
    node_container: NodeContainer<UINodeInput,UINodeOutput>,
    node_animations: SparseSecondaryMap<Node,ElementAnimations>,
    focus_element: Option<Node>,
    captured_element: Option<Node>,
    texture_binds: SlotMap<TextureReference,TTexture>,
}

pub struct UIContext {
    mouse_state: MouseState,
    mouse_position: (f32,f32),
    root_size: (u32,u32),
    root_origin: (u32,u32)
}

impl Default for UIContext {
    fn default() -> Self {
        Self {
            mouse_state: MouseState::Released,
            mouse_position: (f32::MIN,f32::MIN),
            root_size: (1,1),
            root_origin: (0,0)
        }
    }
}

impl<TTexture> Page<TTexture> {

    pub fn create_with_capacity(capacity: usize) -> Self {
        return Self {
            node_container: NodeContainer::create(usize::max(capacity,MINIMUM_NODE_CAPACITY)),
            focus_element: None,
            captured_element: None,
            node_animations: Default::default(),
            texture_binds: Default::default(),
        }
    }

    pub fn move_node_to_parent(&mut self,child: Node,parent: Node) -> Result<(),NodeError> {
        return self.node_container.set_parent(child,parent);
    }

    pub fn move_node_to_root(&mut self,child: Node) -> Result<(),NodeError> {
        return self.node_container.set_parent_root(child);
    }

    pub fn remove_node(&mut self,node: Node) -> Result<(),NodeError> {
        return self.node_container.remove(node);
    }

    pub fn create_node(&mut self,node: UINodeInput) -> Node {
        return self.node_container.insert(node);
    }

    pub fn bind_texture(&mut self,texture: TTexture) -> TextureReference {
        return self.texture_binds.insert(texture);
    }

    pub fn update(&mut self,context: UIContext) -> SmallVec<[PageEvent;8]> {
        return SmallVec::with_capacity(0);
    }

    // TODO: Interface the renderer here - use LocateTexture
}
