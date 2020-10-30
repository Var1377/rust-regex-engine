use super::config::*;
use super::node::*;

#[derive(Clone, Debug)]
pub struct Regex {
    pub expr: String,
    pub(crate) node_vec: Vec<Node>,
    pub config: RegexConfig,
}

impl Default for Regex {
    fn default() -> Self {
        let vec = vec![Node::new_transition()];
        return Regex {
            expr: String::new(),
            node_vec: vec,
            config: RegexConfig::default(),
        };
    }
}

impl Regex {
    pub fn new(regex: &str) -> Self {
        let mut r = Self::default();
        r.expr = regex.to_string();
        r.parse_expression();
        // r.compile();
        return r;
    }

    pub fn new_with_config(regex: &str, config: RegexConfig) -> Self {
        let mut r = Self::new(regex);
        r.config = config;
        return r;
    }
}