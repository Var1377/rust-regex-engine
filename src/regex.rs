use super::compiled_node::*;
use super::node::*;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Regex {
    pub expr: String,
    pub(crate) tree: Option<NodeMap>,
    pub(crate) compiled_root: Option<Rc<CompiledNode>>,
}

impl Default for Regex {
    fn default() -> Self {
        let mut map = NodeMap::default();
        map.insert(0, Node::new_transition());
        return Regex {
            expr: String::from(""),
            tree: Some(map),
            compiled_root: None,
        };
    }
}
