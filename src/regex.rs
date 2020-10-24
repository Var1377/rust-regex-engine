use super::node::*;
use std::rc::Rc;
use super::compiled_node::*;

#[derive(Clone, Debug)]
pub struct Regex {
    pub expr: String,
    pub tree: Option<NodeMap>,
    pub compiled_root: Option<Rc<CompiledNode>>
}

impl Default for Regex {
    fn default() -> Self {
        let mut map = NodeMap::new();
        map.insert(0, Node::new_transition());
        return Regex {
            expr: String::from(""),
            tree: Some(map),
            compiled_root: None
        };
    }
}

