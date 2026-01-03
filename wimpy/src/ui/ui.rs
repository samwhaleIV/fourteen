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

#[derive(PartialEq,Eq)]
pub enum NodeTraversalResult {
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

    fn set_parent(&mut self,child: Node,parent: Node) -> NodeTraversalResult {
        if child == parent {
            return NodeTraversalResult::CircularReference(child);
        }
        if !self.nodes.contains_key(child) {
            return NodeTraversalResult::NullChildReference(child);
        }
        if !self.nodes.contains_key(parent) {
            return NodeTraversalResult::NullParentReference(parent);
        }

        todo!();

        return NodeTraversalResult::Success;
    }

    /* No recursion, no stacks, no heaps... Clean iteration, just the way God intended. */
    fn traverse<F: FnMut(Node) -> NodeTraversalResult>(topology: &SecondaryMap<Node,NodeTopology>,mut node: Option<Node>,mut iterator: F) -> NodeTraversalResult {
        /* DFS traversal using back-tracking parent pointers. */
        loop {
            {
                let Some(node_value) = node else {
                    break; /* Main loop break point. */
                };
                let Some(topology) = topology.get(node_value) else {
                    return NodeTraversalResult::NullReference(node_value);
                };
                let iterator_result = iterator(node_value);
                if iterator_result != NodeTraversalResult::Success {
                    return iterator_result;
                }
                if let Some(child) = topology.first_child {
                    node = Some(child);
                    continue;
                };
            }
            loop {
                let Some(node_value) = node else {
                    break;
                };
                let Some(topology) = topology.get(node_value) else {
                    return NodeTraversalResult::NullReference(node_value);
                };
                if topology.right_sibling.is_some() {
                    break;
                }
                node = topology.parent;
            }
            {
                let Some(node_value) = node else {
                    break;
                };
                let Some(topology) = topology.get(node_value) else {
                    return NodeTraversalResult::NullReference(node_value);
                };
                node = topology.right_sibling;
            }
        }
        return NodeTraversalResult::Success;
    }
/*
    Hey! Where's 'self.topology.remove()'? According to 'SecondaryMap::remove':

    "It's not necessary to remove keys deleted from the primary slot map,
    they get deleted automatically when their slots are reused on a subsequent insert."
    
    In other words, topology data shouldn't turn stale while we are only removing 'self.nodes';
    So, we safely use "expired" topology data throughout this function.

    For more information, see: https://docs.rs/slotmap/1.1.1/slotmap/secondary/struct.SecondaryMap.html
*/
    fn remove(&mut self,root_node: Node) -> NodeTraversalResult {
        /* Validation */
        if self.nodes.get(root_node).is_none() {
            return NodeTraversalResult::NullReference(root_node);
        }
        let Some(root_topology) = self.topology.remove(root_node) else {
            return NodeTraversalResult::MissingTopology(root_node);
        };

        /* If parent exists. */
        if let Some(parent) = root_topology.parent {
            let Some(parent_topology) = self.topology.get_mut(parent) else {
                return NodeTraversalResult::MissingTopology(parent);
            };

            /* If this node is the first child of its parent. */
            if let Some(first_child) = parent_topology.first_child && first_child == root_node {
                // Note: 't.left_sibling' should be 'None' here.
                parent_topology.first_child = root_topology.right_sibling;
            }

            /* Don't use 'else if' because 'last_child' and 'first_child' might both be set. */

            /* If this node is the last child of its parent. */
            if let Some(last_child) = parent_topology.last_child && last_child == root_node {
                // Note: 't.right_sibling' should be 'None' here.
                parent_topology.last_child = root_topology.left_sibling;
            }
        };

        /* If left sibling exists. */
        if let Some(left_sibling) = root_topology.left_sibling {
            let Some(left_sibling_topology) = self.topology.get_mut(left_sibling) else {
                return NodeTraversalResult::MissingTopology(left_sibling);
            };
            /* Pass our right sibling left. */
            left_sibling_topology.right_sibling = root_topology.right_sibling;
        }

        /* If right sibling exists. */
        if let Some(right_sibling) = root_topology.right_sibling {
            let Some(right_sibling_topology) = self.topology.get_mut(right_sibling) else {
                return NodeTraversalResult::MissingTopology(right_sibling);
            };
            /* Pass our left sibling right. */
            right_sibling_topology.left_sibling = root_topology.right_sibling;
        }

        let iterator = |node| match self.nodes.remove(node) {
            Some(_) => NodeTraversalResult::Success,
            None => NodeTraversalResult::NullReference(node),
        };

        return Self::traverse(&self.topology,Some(root_node),iterator);
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

    pub fn move_node_to_parent(&mut self,child: Node,parent: Node) -> NodeTraversalResult {
        return self.node_container.set_parent(child,parent);
    }

    pub fn move_node_to_root(&mut self,child: Node) -> NodeTraversalResult {
        return self.node_container.set_parent(child,self.root_node);
    }

    pub fn remove_node(&mut self,node: Node) -> NodeTraversalResult {
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
