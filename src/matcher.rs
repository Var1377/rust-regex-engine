use super::backtrack_matcher::*;
use super::config::*;
use super::nfa::*;
use super::parallel_nfa;
use super::regex::*;
use super::utils::*;
use std::ops::DerefMut;

impl Regex {
    pub fn is_match(&self, string: &str) -> bool {
        match self.engine.lock().unwrap().deref_mut() {
            MatchingEngine::Backtrack { callstack, backref_data: _ } => {
                // let chars = string.chars().collect::<Vec<_>>();
                // return c_pure_match(&self.node_vec, &chars, callstack, self.root_node_idx);
                return backtrack_pure_match(&self.node_vec, string.as_bytes(), self.root_node_idx, callstack, &self.optimized_root_node);
            }
            MatchingEngine::ParallelNFA {} => {
                return parallel_nfa::pure_match(&self.node_vec, string.as_bytes(), self.root_node_idx);
            }
            _ => unimplemented!(),
        };
    }

    pub fn match_str(&self, string: &str) -> bool {
        return self.is_match(string);
    }

    pub fn first_match(&self, string: &str) -> Option<(usize, usize)> {
        match self.engine.lock().unwrap().deref_mut() {
            MatchingEngine::Backtrack { callstack, backref_data: _ } => {
                return backtrack_first_match(&self.node_vec, string.as_bytes(), self.root_node_idx, callstack, &self.optimized_root_node);
            }
            MatchingEngine::ParallelNFA {} => {
                return parallel_nfa::index_match(&self.node_vec, string.as_bytes(), self.root_node_idx);
            }
            _ => unimplemented!(),
        };
    }

    pub fn match_indices(&self, string: &str) -> Vec<(usize, usize)> {
        match self.engine.lock().unwrap().deref_mut() {
            MatchingEngine::Backtrack { callstack, backref_data: _ } => {
                // let chars = string.char_indices().collect::<Vec<_>>();
                // return c_indices_match(&self.node_vec, &chars, callstack, self.root_node_idx)
                return backtrack_match_indices(&self.node_vec, string.as_bytes(), self.root_node_idx, callstack, &self.optimized_root_node);
            }
            MatchingEngine::ParallelNFA {} => {
                return parallel_nfa::indices_match(&self.node_vec, string.as_bytes(), self.root_node_idx);
            }
            _ => unimplemented!(),
        };
    }
}
