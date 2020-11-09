use super::config::*;
use super::iterative_match::*;
use super::nfa::*;
use super::recursive_match::*;
use super::regex::*;
use super::utils::*;

impl Regex {
    pub fn match_str(&self, string: &str) -> bool {
        let chars = str_to_char_vec(string);
        match self.config.location {
            SearchLocation::Global => {
                for i in 0..chars.len() {
                    if iterative_pure_match(&self.node_vec, &chars, i) {
                        return true;
                    }
                }
                return false;
            }
            SearchLocation::First => {
                return iterative_pure_match(&self.node_vec, &chars, 0);
            }
            SearchLocation::Sticky(index) => {
                return iterative_pure_match(&self.node_vec, &chars, index);
            }
        }
    }

    pub fn match_str_iter(&self, string: &str) -> bool {
        let chars = str_to_char_vec(string);
        match self.config.location {
            SearchLocation::Global => {
                for i in 0..chars.len() {
                    if iterative_pure_match(&self.node_vec, &chars, i) {
                        return true;
                    }
                }
                return false;
            }
            SearchLocation::First => {
                return iterative_pure_match(&self.node_vec, &chars, 0);
            }
            SearchLocation::Sticky(index) => {
                return iterative_pure_match(&self.node_vec, &chars, index);
            }
        }
    }

    pub fn match_str_recursive(&self, string: &str) -> bool {
        let chars = str_to_char_vec(string);
        match self.config.location {
            SearchLocation::Global => {
                for i in 0..chars.len() {
                    if recursive_pure_match(&self.node_vec, &chars, &0, i) {
                        return true;
                    }
                }
                return false;
            }
            SearchLocation::First => {
                return recursive_pure_match(&self.node_vec, &chars, &0, 0);
            }
            SearchLocation::Sticky(i) => {
                return recursive_pure_match(&self.node_vec, &chars, &0, i);
            }
        }
    }

    pub fn match_string(&self, string: String) -> bool {
        return self.match_str(string.as_str());
    }
}
