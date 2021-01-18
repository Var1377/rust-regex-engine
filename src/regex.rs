use super::compiled_node::CompiledNode;
use super::config::*;
use super::nfa::*;
use crate::root_node_optimizer::RootNode;
use std::alloc;
use std::sync::Mutex;

#[derive(Debug)]
pub struct Regex {
    pub expr: String,
    pub(crate) node_vec: Vec<CompiledNode>,
    pub(crate) root_node_idx: usize,
    pub(crate) optimized_root_node: Option<RootNode>,
    pub(crate) engine: Mutex<MatchingEngine>,
    pub(crate) anchored: bool,
}

impl Clone for Regex {
    fn clone(&self) -> Self {
        return Self::new(&self.expr);
    }
}

impl Regex {
    // Not default becuase we don't want it to be public
    fn base() -> Self {
        return Regex {
            expr: String::new(),
            node_vec: vec![],
            root_node_idx: 0,
            optimized_root_node: None,
            engine: Mutex::new(MatchingEngine::default()),
            anchored: false,
        };
    }

    pub fn new(regex: &str) -> Self {
        let mut r = Self::base();
        r.expr = regex.to_string();
        r.parse_expression();
        return r;
    }
}

// struct RegexSet {
//     // not exactly sure what the plural of regex is...
//     pub regexes: Vec<Regex>,
// }

// impl RegexSet {
//     fn new(regexes: Vec<Regex>) -> Self {
//         return Self { regexes };
//     }
// }

use fxhash::FxHashMap;

#[derive(Copy, Debug, Clone, PartialEq)]
pub enum EngineFlag {
    Backtrack,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MatchingEngine {
    Backtrack {
        callstack: Vec<(usize, usize, usize)>,
        backref_data: FxHashMap<String, u32>,
    },
    HybridAutomata {},
    ParallelNFA {},
    OnlineDFA {},
    PrecompiledDFA {},
}

use super::utils::str_to_char_vec;
use super::*;

impl Default for MatchingEngine {
    fn default() -> Self {
        return Self::Backtrack {
            callstack: vec![],
            backref_data: FxHashMap::default(),
        };
    }
}
