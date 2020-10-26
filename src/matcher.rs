use super::node::*;
use super::regex::*;
use super::utils::*;
impl Regex {
    pub fn match_str(&self, string: &str) -> bool {
        fn _match(node_vec: &[Node], chars: &[char], node_index: &usize, char_index: usize) -> bool {
            let node: &Node;
            unsafe {node = node_vec.get_unchecked(node_index.clone());};
            match node {
                Node::Transition { children, .. } => {
                    for child in children {
                        if _match(node_vec, chars, child, char_index) {
                            return true;
                        }
                    }
                    return false;
                }
                Node::Inclusive { characters, children, .. } => {
                    if char_index == chars.len() {
                        return false;
                    }
                    let to_match = chars[char_index];
                    if characters.contains(&to_match) {
                        for child in children {
                            if _match(node_vec, chars, child, char_index + 1) {
                                return true;
                            }
                        }
                        return false;
                    } else {
                        return false;
                    }
                }
                Node::End => {
                    return true;
                }
                Node::Exclusive { characters, children, .. } => {
                    if char_index == chars.len() {
                        return false;
                    }
                    let to_match = chars[char_index];
                    if characters.contains(&to_match) {
                        false
                    } else {
                        for child in children {
                            if _match(node_vec, chars, child, char_index + 1) {
                                return true;
                            }
                        }
                        return false;
                    }
                }
                Node::MatchAll { children, .. } => {
                    if char_index == chars.len() {
                        return false;
                    }
                    let to_match = chars[char_index];
                    if to_match == '\n' {
                        return false;
                    } else {
                        for child in children {
                            if _match(node_vec, chars, child, char_index + 1) {
                                return true;
                            }
                        }
                        return false;
                    }
                }
                Node::EndOfLine { children, .. } => {
                    if char_index == chars.len() {
                        for child in children {
                            if _match(node_vec, chars, child, char_index) {
                                return true;
                            }
                        }
                        return false;
                    }
                    if chars[char_index] == '\n' {
                        {
                            for child in children {
                                if _match(node_vec, chars, child, char_index + 1) {
                                    return true;
                                }
                            }
                            return false;
                        }
                    }
                    return false;
                }
                Node::BeginningOfLine { children, .. } => {
                    if char_index == 0 {
                        for child in children {
                            if _match(node_vec, chars, child, char_index) {
                                return true;
                            }
                        }
                        return false;
                    }
                    if chars[char_index] == '\n' {
                        {
                            for child in children {
                                if _match(node_vec, chars, child, char_index + 1) {
                                    return true;
                                }
                            }
                            return false;
                        }
                    }
                    return false;
                }
                Node::MatchOne { character, children } => {
                    if char_index == chars.len() {
                        return false;
                    }
                    let to_match = chars[char_index];
                    if to_match == *character {
                        for child in children {
                            if _match(node_vec, chars, child, char_index + 1) {
                                return true;
                            }
                        }
                        return false;
                    }
                    return false;
                }
                Node::NotMatchOne { character, children } => {
                    if char_index == chars.len() {
                        return false;
                    }
                    let to_match = chars[char_index];
                    if to_match != *character {
                        for child in children {
                            if _match(node_vec, chars, child, char_index + 1) {
                                return true;
                            }
                        }
                        return false;
                    }
                    return false;
                }
            }
        }
        let chars = str_to_char_vec(string);
        for i in 0..chars.len() {
            if _match(&self.node_vec, &chars, &0, i) {
                return true;
            }
        }
        return false;
    }

    pub fn match_string(&self, string: String) -> bool {
        return self.match_str(string.as_str());
    }
}
