use super::backtrack_matcher::*;
use super::config::*;
use super::nfa::*;
use super::regex::*;
use super::utils::*;

impl Regex {
    pub fn match_str(&self, string: &str) -> bool {
        // Make it here to avoid contant memory reallocation for every matching attempt
        // It's cleared after every matching attempt
        let ref s = str_to_char_vec(string);
        let mut callstack: Vec<(usize, usize, usize, bool)> = Vec::with_capacity(self.node_vec.len() * 4);
        for i in 0..s.len() {
            if pure_match(&self.node_vec, &s, i, &mut callstack) {
                return true;
            }
        }
        return false;
    }

    pub fn match_string(&self, string: String) -> bool {
        return self.match_str(string.as_str());
    }
}
