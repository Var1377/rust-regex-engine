use super::backtrack_matcher::*;
use super::config::*;
use super::nfa::*;
use super::regex::*;
use super::utils::*;
use std::ops::DerefMut;

impl Regex {
    pub fn match_str(&self, string: &str) -> bool {
        // Make it here to avoid contant memory reallocation for every matching attempt
        // It's cleared after every matching attempt
        let ref s = str_to_char_vec(string);
        return self.match_chars(s);
    }

    pub fn match_string(&self, string: String) -> bool {
        return self.match_str(string.as_str());
    }

    pub fn match_chars(&self, chars: &[char]) -> bool {
        match self.engine.lock().unwrap().deref_mut() {
            MatchingEngine::Backtrack {callstack, backref_data} => {
                for i in 0..chars.len() {
                    if c_pure_match(&self.node_vec, chars, callstack, i, self.root_node).is_some() {
                        return true;
                    }
                }
                return false;
            },
            _ => unimplemented!(),
        };
    }

    pub fn first_captures_chars(&self, chars: &[char]) -> Option<CapturesMap> {
        match self.engine.lock().unwrap().deref_mut() {
            MatchingEngine::Backtrack {callstack, backref_data} => {
                for i in 0..chars.len() {
                    let map = c_captures_match(&self.node_vec, chars, callstack, i, self.root_node);
                    if map.is_some() { return map }
                }
                return None;
            },
            _ => unimplemented!(),
        };
    }

    pub fn first_captures_string(&self, string: &str) -> Option<CapturesMap> {
        return self.first_captures_chars(&string.chars().collect::<Vec<_>>())
    }

    pub fn match_indices_chars(&self, chars: &[char]) -> Vec<(usize, usize)> {
        let mut output = Vec::new();

        match self.engine.lock().unwrap().deref_mut() {
            MatchingEngine::Backtrack {callstack, backref_data} => {
                let mut i = 0;
                while i < chars.len() {
                    let res =  c_pure_match(&self.node_vec, chars, callstack, i, self.root_node);
                    match res {
                        Some(end) => {
                            println!("Found a match at {}, {}", i, end);
                            output.push((i, end));
                            i = end;
                        }, 
                        None => ()
                    };
                    i += 1;
                }
                return output;
            },
            _ => unimplemented!(),
        };
    }
}