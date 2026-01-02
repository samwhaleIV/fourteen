const MINIMUM_NODE_CAPACITY: usize = 32; /* Should be at least 1. */

use std::{
    time::{
        Duration,
        Instant
    }
};

use slotmap::{
    SecondaryMap,
    SlotMap,
    SparseSecondaryMap
};

use crate::{
    shared::{
        Area,
        Color,
        Layout
    },
    wgpu::Frame
};

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
    pub struct Node;
    pub struct TextureReference;
}

#[derive(Default,Clone,Copy)]
struct NodeTopology {
    parent: Option<Node>,
    first_child: Option<Node>,
    last_child: Option<Node>,
    left_sibling: Option<Node>,
    right_sibling: Option<Node>
}

struct NodeData {
    layout: Layout,
    uv: Area,
    rotation: f32,
    color: Color,
    texture: Option<TextureReference>,
    interaction_mode: InteractionMode
}

impl Default for NodeData {
    fn default() -> Self {
        Self {
            layout: Default::default(),
            uv: Area::ONE,
            rotation: 0.0,
            color: Color::WHITE,
            texture: None,
            interaction_mode: InteractionMode::None
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

pub enum NodeManipulationResult {
    Success,
    NullReference(Node),
    NullChildReference(Node),
    NullParentReference(Node),
    CircularReference(Node),
    MissingTopology(Node),
}

pub enum MouseState {
    Released,
    JustPressed,
    Held,
    JustReleased,
}

pub struct LayoutComputationContext {
    mouse_state: MouseState,
    mouse_position: (f32,f32),
    root_size: (u32,u32),
    root_origin: (u32,u32)
}

impl Default for LayoutComputationContext {
    fn default() -> Self {
        Self {
            mouse_state: MouseState::Released,
            mouse_position: (f32::MIN,f32::MIN),
            root_size: (1,1),
            root_origin: (0,0)
        }
    }
}

struct NodeContainer {
    nodes: SlotMap<Node,NodeData>,
    topology: SecondaryMap<Node,NodeTopology>,
}

impl NodeContainer {

    fn set_parent(&mut self,child: Node,parent: Node) -> NodeManipulationResult {
        if child == parent {
            return NodeManipulationResult::CircularReference(child);
        }
        if !self.nodes.contains_key(child) {
            return NodeManipulationResult::NullChildReference(child);
        }
        if !self.nodes.contains_key(parent) {
            return NodeManipulationResult::NullParentReference(parent);
        }

        todo!();

        return NodeManipulationResult::Success;
    }

    fn remove_descendants(&mut self,start: NodeTopology) {
        let Some(mut node) = start.first_child else {
            /* Early return, node does not have children. */
            return;
        };
        loop {
            let Some(topology) = self.topology.remove(node) else {
                /* Topology not found. Shouldn't usually happen. */
                break;
            };

            self.nodes.remove(node);
            self.remove_descendants(topology);

            let Some(sibling) = topology.right_sibling else {
                /* No siblings remain. */
                break;
            };

            /* Set the node for the next iteration. */
            node = sibling; 
        }
    }

    fn remove(&mut self,node: Node) -> NodeManipulationResult {
        /* Validation */
        if self.nodes.remove(node).is_none() {
            return NodeManipulationResult::NullReference(node);
        }
        let Some(t) = self.topology.remove(node) else {
            return NodeManipulationResult::MissingTopology(node);
        };

        /* If parent exists. */
        if let Some(parent) = t.parent {
            let Some(pt) = self.topology.get_mut(parent) else {
                return NodeManipulationResult::MissingTopology(parent);
            };

            /* If this node is the first child of its parent. */
            if let Some(first_child) = pt.first_child && first_child == node {
                // Note: 't.left_sibling' should be 'None' here.
                pt.first_child = t.right_sibling;
            }

            /* Don't use 'else if' because 'last_child' and 'first_child' might both be set. */

            /* If this node is the last child of its parent. */
            if let Some(last_child) = pt.last_child && last_child == node {
                // Note: 't.right_sibling' should be 'None' here.
                pt.last_child = t.left_sibling;
            }
        };

        /* If left sibling exists. */
        if let Some(left_sibling) = t.left_sibling {
            let Some(lt) = self.topology.get_mut(left_sibling) else {
                return NodeManipulationResult::MissingTopology(left_sibling);
            };
            /* Pass our right sibling left. */
            lt.right_sibling = t.right_sibling;
        }

        /* If right sibling exists. */
        if let Some(right_sibling) = t.right_sibling {
            let Some(rt) = self.topology.get_mut(right_sibling) else {
                return NodeManipulationResult::MissingTopology(right_sibling);
            };
            /* Pass our left sibling right. */
            rt.left_sibling = t.right_sibling;
        }

        self.remove_descendants(t);

        return NodeManipulationResult::Success;
    }
}

pub struct Page<TTexture> {
    node_container: NodeContainer,
    node_animations: SparseSecondaryMap<Node,ElementAnimations>,
    root_node: Node,
    focus_element: Option<Node>,
    captured_element: Option<Node>,
    texture_binds: SlotMap<TextureReference,TTexture>,
}

impl<TTexture> Page<TTexture> {

    pub fn create_with_capacity(capacity: usize) -> Self {
        let capacity = usize::max(capacity,MINIMUM_NODE_CAPACITY);

        let mut nodes = SlotMap::with_capacity_and_key(capacity);
        let mut topology = SecondaryMap::with_capacity(capacity);

        let root_node = nodes.insert(NodeData::default());
        topology.insert(root_node,NodeTopology::default());


        let node_container = NodeContainer {
            nodes,
            topology
        };

        return Self {
            node_container,
            root_node,
            focus_element: None,
            captured_element: None,
            node_animations: Default::default(),
            texture_binds: Default::default(),
        }
    }

    pub fn move_node_to_parent(&mut self,child: Node,parent: Node) -> NodeManipulationResult {
        return self.node_container.set_parent(child,parent);
    }

    pub fn move_node_to_root(&mut self,child: Node) -> NodeManipulationResult {
        return self.node_container.set_parent(child,self.root_node);
    }

    pub fn remove_node(&mut self,node: Node) -> NodeManipulationResult {
        return self.node_container.remove(node);
    }

    pub fn create_node(&mut self,node: NodeData) -> Node {
        return self.node_container.nodes.insert(node);
    }

    pub fn bind_texture(&mut self,texture: TTexture) -> TextureReference {
        return self.texture_binds.insert(texture);
    }

    pub fn compute(&self,context: &LayoutComputationContext) -> ComputedLayout {
        return context.compute(self);
    }
}

struct ComputedLayout {
    //todo...
}

impl LayoutComputationContext {
    pub fn compute<TTexture>(&self,_page: &Page<TTexture>) -> ComputedLayout {
        todo!();
    }
}

impl ComputedLayout {
    pub fn render(&self,_frame: Frame) {
        todo!();
    }
}
