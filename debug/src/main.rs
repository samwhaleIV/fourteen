use wimpy::ui::*;
use std::fmt::{
    Formatter,
    Display,
    Result
};

#[derive(Default,Debug)]
struct NodeInput(u64);

#[derive(Default,Debug)]
struct NodeOutput(u64);

impl Display for NodeOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f,"{}",self.0)
    }
}

fn main() {
    let mut node_container = NodeContainer::<NodeInput,NodeOutput>::create(128);

    let node1a = node_container.insert(NodeInput(101));
    let node2a = node_container.insert(NodeInput(102));
    let node3a = node_container.insert(NodeInput(103));
    let node4a = node_container.insert(NodeInput(104));


    let node1b = node_container.insert(NodeInput(101));
    let node2b = node_container.insert(NodeInput(102));
    let node3b = node_container.insert(NodeInput(103));
    let node4b = node_container.insert(NodeInput(104));

    node_container.set_parent(node4a,node3a).expect("Could not set parent");
    node_container.set_parent(node3a,node2a).expect("Could not set parent");
    node_container.set_parent(node2a,node1a).expect("Could not set parent");
    node_container.set_parent_root(node1a).expect("Could not set parent");

    node_container.set_parent(node1b,node3a).expect("Could not set parent");
    node_container.set_parent(node2b,node3a).expect("Could not set parent");
    node_container.set_parent(node3b,node3a).expect("Could not set parent");
    node_container.set_parent(node4b,node3a).expect("Could not set parent");

    node_container.update_flat_map().expect("Could not update flat map");

    node_container.print_flat_map();
}
