use slotmap::{
    SlotMap,
    SecondaryMap,
};
use std::{
    hash::Hash,
    fmt::Display,
};

slotmap::new_key_type! {
    pub struct Node;
}

#[derive(Default,Clone,Copy)]
struct NodeTopology {
    parent: Option<Node>,
    leftmost_child: Option<Node>,
    rightmost_child: Option<Node>,
    left_sibling: Option<Node>,
    right_sibling: Option<Node>,
}

#[derive(PartialEq,Eq,Debug)]
pub enum NodeError {
    NullReference(Node),
    CircularReference(Node),
    MissingTopology(Node),
    FlatMapBacktrackFatality
}

type TopologyMap = SecondaryMap<Node,NodeTopology>;

pub struct NodeContainer<TInput,TOutput>{
    nodes: SlotMap<Node,TInput>,
    topology: TopologyMap,
    root_node: Node,
    flat_map: Vec<NodeFlatMapItem<TOutput>>,
    flat_map_is_valid: bool,
}
enum BranchControl {
    Start,
    End
}
struct FlatMapBranchControl {
    source_index: usize,
    change: BranchControl
}

enum NodeFlatMapItem<TOutput> {
    BranchControl(FlatMapBranchControl),
    Cache(NodeFlatMapData<TOutput>)
}

impl<TOutput> NodeFlatMapItem<TOutput> {
    pub fn start_branch(source_index: usize) -> Self {
        return Self::BranchControl(FlatMapBranchControl {
            source_index,
            change: BranchControl::Start
        });
    }
    pub fn end_branch(source_index: usize) -> Self {
        return Self::BranchControl(FlatMapBranchControl {
            source_index,
            change: BranchControl::End
        });
    }
}

struct NodeFlatMapData<TOutput> {
    identity: Node,
    parent_index: usize,
    value: TOutput
}

trait TopologyMapExtension {
    fn traverse_nodes_dfs<F>(
        &self,
        root_node: Node,
        iterator: F
    ) -> Result<(),NodeError> where F: FnMut(Node) -> Result<(),NodeError>;

    fn fill_flat_map_dfs<TOutput>(
        &self,root_node: Node,
        flat_map: &mut Vec<NodeFlatMapItem<TOutput>>
    ) -> Result<(),NodeError> where TOutput: Default;

    fn unbind_parent_relationship(
        &mut self,
        node: Node
    ) -> Result<(),NodeError>;

    fn bind_parent_relationship(
        &mut self,
        child: Node,
        parent: Node
    ) -> Result<(),NodeError>;
}

pub trait NodeOutputBuilder<TInput,TOutput> {
    fn clear(&mut self);

    fn start_branch(&mut self,input: &TInput,branch_cache: &TOutput);
    fn end_branch(&mut self,input: &TInput,branch_cache: &TOutput);

    fn next(&mut self,input: &TInput,parent_cache: &TOutput) -> TOutput;
}

impl TopologyMapExtension for TopologyMap {

    fn traverse_nodes_dfs<F>(&self,root: Node,mut iterator: F) -> Result<(),NodeError> where F: FnMut(Node) -> Result<(),NodeError> {
        /* DFS traversal using back-tracking parent pointers. */
        let mut node = Some(root);
        'node_visitor: loop {
            {
                let Some(node_value) = node else { break 'node_visitor; };
                let Some(topology) = self.get(node_value) else { return Err(NodeError::MissingTopology(node_value)); };

                iterator(node_value)?;

                if let Some(child) = topology.leftmost_child {
                    node = Some(child);
                    continue 'node_visitor;
                };
            }
            'parent_backtrack: loop {
                let Some(node_value) = node else { break 'node_visitor; };
                let Some(topology) = self.get(node_value) else { return Err(NodeError::MissingTopology(node_value)); };

                if topology.right_sibling.is_some() {
                    node = topology.right_sibling;
                    break 'parent_backtrack;
                }

                node = topology.parent;
            }
        }
        return Ok(());
    }

    fn fill_flat_map_dfs<TOutput>(&self,root: Node,flat_map: &mut Vec<NodeFlatMapItem<TOutput>>) -> Result<(),NodeError> where TOutput: Default {
        let mut parent_index: usize = flat_map.len();
        let mut node = Some(root);
        'node_visitor: loop {
            {
                let Some(node_value) = node else { break 'node_visitor; };
                let Some(topology) = self.get(node_value) else { return Err(NodeError::MissingTopology(node_value)); };

                let current_index = flat_map.len();
                flat_map.push(NodeFlatMapItem::Cache(NodeFlatMapData{
                    identity: node_value,
                    parent_index,
                    value: Default::default()
                }));
                flat_map.push(NodeFlatMapItem::start_branch(current_index));
                parent_index = current_index;
                if let Some(child) = topology.leftmost_child {
                    node = Some(child);
                    continue 'node_visitor;
                };
            }
            'parent_backtrack: loop {
                let Some(node_value) = node else { break 'node_visitor; };
                let Some(topology) = self.get(node_value) else { return Err(NodeError::MissingTopology(node_value)); };

                flat_map.push(NodeFlatMapItem::end_branch(parent_index));
                parent_index = match &flat_map[parent_index] {
                    NodeFlatMapItem::Cache(output) => output.parent_index,
                    _ => return Err(NodeError::FlatMapBacktrackFatality)
                };

                if topology.right_sibling.is_some() {
                    node = topology.right_sibling;
                    break 'parent_backtrack;
                }

                node = topology.parent;
            }
        }
        return Ok(());
    }

    fn unbind_parent_relationship(&mut self,node: Node) -> Result<(),NodeError> {
        let (parent,left_sibling,right_sibling) = match self.get(node) {
            Some(t) => (t.parent,t.left_sibling,t.right_sibling),
            None => return Err(NodeError::MissingTopology(node))
        };

        let Some(parent) = parent else {
            return Ok(());
        };

        let Some(parent_topology) = self.get_mut(parent) else {
            return Err(NodeError::MissingTopology(parent));
        };

        if let Some(first_child) = parent_topology.leftmost_child && first_child == node {
            parent_topology.leftmost_child = right_sibling;
        }

        if let Some(last_child) = parent_topology.rightmost_child && last_child == node {
            parent_topology.rightmost_child = left_sibling;
        }

        if let Some(left_sibling) = left_sibling {
            let Some(left_sibling_topology) = self.get_mut(left_sibling) else {
                return Err(NodeError::MissingTopology(left_sibling));
            };
            left_sibling_topology.right_sibling = right_sibling;
        }

        if let Some(right_sibling) = right_sibling {
            let Some(right_sibling_topology) = self.get_mut(right_sibling) else {
                return Err(NodeError::MissingTopology(right_sibling));
            };
            right_sibling_topology.left_sibling = left_sibling;
        }

        let Some(node_topology) = self.get_mut(node) else {
            return Err(NodeError::MissingTopology(node));
        };

        node_topology.parent = None;
        node_topology.left_sibling = None;
        node_topology.right_sibling = None;

        return Ok(());
    }

    fn bind_parent_relationship(&mut self,child: Node,parent: Node) -> Result<(),NodeError> {
        let Some(parent_topology) = self.get_mut(parent) else {
            return Err(NodeError::MissingTopology(parent));
        };
        if let Some(last_child) = parent_topology.rightmost_child {
            parent_topology.rightmost_child = Some(child);
            let Some(last_child_topology) = self.get_mut(last_child) else {
                return Err(NodeError::MissingTopology(last_child));
            };
            last_child_topology.right_sibling = Some(child);
            let Some(child_topology) = self.get_mut(child) else {
                return Err(NodeError::MissingTopology(child));
            };
            child_topology.left_sibling = Some(last_child);
            child_topology.right_sibling = None;
            child_topology.parent = Some(parent);
        } else {
            parent_topology.leftmost_child = Some(child);
            parent_topology.rightmost_child = Some(child);
            let Some(child_topology) = self.get_mut(child) else {
                return Err(NodeError::MissingTopology(child));
            };
            child_topology.left_sibling = None;
            child_topology.right_sibling = None;
            child_topology.parent = Some(parent);
        }
        return Ok(());
    }
}

impl<TInput,TOutput> NodeContainer<TInput,TOutput> where TInput: Default, TOutput: Default {
    pub fn create(capacity: usize) -> Self {
        let mut nodes = SlotMap::with_capacity_and_key(capacity);
        let mut topology = SecondaryMap::with_capacity(capacity);

        let root_node = nodes.insert(TInput::default());
        topology.insert(root_node,NodeTopology::default());

        let flat_map = Vec::with_capacity((capacity + 1) * 3);

        return Self {
            nodes,
            topology,
            root_node,
            flat_map,
            flat_map_is_valid: false,
        }
    }

    pub fn insert(&mut self,node_value: TInput) -> Node {
        // TODO: resize buffers ahead of time
        let node = self.nodes.insert(node_value);
        self.topology.insert(node,NodeTopology::default());
        return node;
    }

    pub fn set_parent(&mut self,child_node: Node,parent_node: Node) -> Result<(),NodeError> {
        if child_node == parent_node {
            return Err(NodeError::CircularReference(child_node));
        }
        if !self.nodes.contains_key(child_node) {
            return Err(NodeError::NullReference(child_node));
        }
        if !self.nodes.contains_key(parent_node) {
            return Err(NodeError::NullReference(parent_node));
        }

        self.flat_map_is_valid = false;

        self.topology.unbind_parent_relationship(child_node)?;
        self.topology.bind_parent_relationship(child_node,parent_node)
    }

    pub fn set_parent_root(&mut self,child_node: Node) -> Result<(),NodeError> {
        return self.set_parent(child_node,self.root_node);
    }

    pub fn remove(&mut self,node: Node) -> Result<(),NodeError> {
        if !self.nodes.contains_key(node) {
            return Err(NodeError::NullReference(node));
        }

        self.flat_map_is_valid = false;

        self.topology.unbind_parent_relationship(node)?;

        self.topology.traverse_nodes_dfs(node,|node| match self.nodes.remove(node) {
            Some(_) => Ok(()),
            None => Err(NodeError::NullReference(node)),
        })
    }

    pub fn update_flat_map(&mut self) -> Result<(),NodeError> {
        if self.flat_map_is_valid {
            return Ok(())
        }
        let flat_map = &mut self.flat_map;
        flat_map.clear();
        self.topology.fill_flat_map_dfs(self.root_node,flat_map)?;
        self.flat_map_is_valid = true;
        return Ok(())
    }

    pub fn update_root_node(&mut self,root_input: TInput) {
        self.nodes[self.root_node] = root_input;
    }

    pub fn build_output(&mut self,mut output_builder: impl NodeOutputBuilder<TInput,TOutput>) {
        debug_assert!(self.flat_map_is_valid);
        output_builder.clear();
        for i in 0..self.flat_map.len() {
            match &self.flat_map[i] {
                NodeFlatMapItem::Cache(NodeFlatMapData {
                    identity, parent_index, ..
                }) => {
                    let Some(input) = self.nodes.get(*identity) else {
                        continue;
                    };
                    let parent_output = match &self.flat_map[*parent_index] {
                        NodeFlatMapItem::Cache(data) => &data.value,
                        _ => continue
                    };
                    self.flat_map[i] = NodeFlatMapItem::Cache(NodeFlatMapData {
                        identity: *identity,
                        parent_index: *parent_index,
                        value: output_builder.next(input,&parent_output)
                    });
                },
                NodeFlatMapItem::BranchControl(FlatMapBranchControl {
                    source_index,
                    change 
                }) => {
                    let (input,output) = match &self.flat_map[*source_index] {
                        NodeFlatMapItem::Cache(data) => match self.nodes.get(data.identity) {
                            Some(input) => (input,&data.value),
                            None => (&Default::default(),&data.value),
                        },
                        _ => (&Default::default(),&Default::default())
                    };
                    match change {
                        BranchControl::Start => output_builder.start_branch(input,output),
                        BranchControl::End => output_builder.end_branch(input,output),
                    }
                }
            }
        }
    }
}

impl<TInput,TOutput> NodeContainer<TInput,TOutput> where TOutput: Default + Display {   
    pub fn print_flat_map(&self) {
        if self.flat_map.len() == 0 {
            println!("Flat map is empty");
            return;
        }
        let mut start = 0;
        for item in &self.flat_map {
            match item {
                NodeFlatMapItem::BranchControl(FlatMapBranchControl {
                    source_index, 
                    change
                }) => println!("{} | {} Index: {}",start,match change {
                    BranchControl::Start => "Branch Start",
                    BranchControl::End => "Branch End",
                },source_index),
                NodeFlatMapItem::Cache(value) =>  println!("{} | Output: {}",start,value.value),
            };
            start += 1;
        }
    }
}
