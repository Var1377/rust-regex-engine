use super::config::*;
use super::nfa::*;
use std::alloc;

#[derive(Clone, Debug, PartialEq)]
pub struct Regex {
    pub expr: String,
    pub(crate) node_vec: Vec<Node>,
    pub(crate) engine: MatchingEngine,
    pub(crate) anchored: bool
}

impl Regex {
    // Not default becuase we don't want it to be public
    fn base() -> Self {
        return Regex {
            expr: String::new(),
            node_vec: vec![Node::new_transition()],
            engine: MatchingEngine::default(),
            anchored: false
        };
    }

    pub fn new(regex: &str) -> Self {
        let mut r = Self::base();
        r.expr = regex.to_string();
        r.parse_expression();
        // r.compile();
        return r;
    }
}

struct RegexSet {
    // not exactly sure what the plural of regex is...
    pub regexes: Vec<Regex>,
}

impl RegexSet {
    fn new(regexes: Vec<Regex>) -> Self {
        return Self {
            regexes
        }
    }
}

use fxhash::FxHashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MatchingEngine {
    Backtrack {
        callstack: Vec<(usize, usize, usize, bool)>,
        nodes: Vec<Node>,
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
            nodes: vec![],
            backref_data: FxHashMap::default()
        }
    }
}

impl MatchingEngine {
    fn pure_match(&mut self, string: &str) -> bool {
        match self {
            Self::Backtrack {
                callstack,
                nodes,
                backref_data,
            } => {
                let ref s = str_to_char_vec(string);
                for i in 0..s.len() {
                    if backtrack_matcher::pure_match(nodes, &s, i, callstack) {
                        return true;
                    }
                }
                false
            }
            Self::ParallelNFA {} => unimplemented!(),
            Self::PrecompiledDFA {} => unimplemented!(),
            Self::OnlineDFA {} => unimplemented!(),
            Self::HybridAutomata {} => unimplemented!(),
        }
    }
}
