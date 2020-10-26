use super::node::*;

#[derive(Clone, Debug)]
pub struct Regex {
    pub expr: String,
    pub(crate) node_vec: Vec::<Node>,
}

impl Default for Regex {
    fn default() -> Self {
        let vec = vec![Node::new_transition()];
        return Regex {
            expr: String::from(""),
            node_vec: vec,
        };
    }
}
