use super::node::*;

#[derive(Clone, Debug)]
pub struct Regex {
    pub expr: String,
    pub(crate) tree: NodeMap,
}

impl Default for Regex {
    fn default() -> Self {
        let mut map = NodeMap::default();
        map.insert(0, Node::new_transition());
        return Regex {
            expr: String::from(""),
            tree: map,
        };
    }
}
