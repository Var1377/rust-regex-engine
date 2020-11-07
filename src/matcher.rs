use super::config::*;
use super::node::*;
use super::regex::*;
use super::utils::*;

impl Regex {
    pub fn match_str(&self, string: &str) -> bool {
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
            SearchLocation::Sticky(index) => {
                return recursive_pure_match(&self.node_vec, &chars, &0, index);
            }
        }
    }

    pub fn match_string(&self, string: String) -> bool {
        return self.match_str(string.as_str());
    }
}

// fn iterative_pure_match(node_vec: &[Node], chars: &[char], start_index: usize) -> bool {
//     // Callstack: Vec<(node_index, child, char_index)>
//     let mut callstack = Vec::with_capacity(node_vec.len());
//     let mut char_index = 0usize;
//     callstack.push((0usize, 0usize, start_index));
//     loop {
//         match callstack.last() {
//             None => return false,
//             Some(x) => {
//                 let (node_index, child) = x;
//                 let node: &Node;
//                 unsafe {
//                     node = node_vec.get_unchecked(*node_index);
//                 }
//                 match node {
//                     Node::Transition{ ref children } => {

//                     },
//                     Node::End => return true,
//                 };
//             }
//         }
//     }
//     true
// }

fn recursive_pure_match(node_vec: &[Node], chars: &[char], node_index: &usize, char_index: usize) -> bool {
    let node: &Node;
    unsafe {
        node = node_vec.get_unchecked(*node_index);
    };
    match node {
        Node::Transition { children, .. } => {
            for child in children {
                if recursive_pure_match(node_vec, chars, child, char_index) {
                    return true;
                }
            }
            return false;
        }
        Node::Inclusive { characters, children, .. } => {
            if char_index == chars.len() {
                return false;
            }
            let to_match: &char;
            unsafe {
                to_match = chars.get_unchecked(char_index);
            }
            if characters.contains(to_match) {
                for child in children {
                    if recursive_pure_match(node_vec, chars, child, char_index + 1) {
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
            let to_match: &char;
            unsafe {
                to_match = chars.get_unchecked(char_index);
            }
            if characters.contains(to_match) {
                false
            } else {
                for child in children {
                    if recursive_pure_match(node_vec, chars, child, char_index + 1) {
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
            let to_match: &char;
            unsafe {
                to_match = chars.get_unchecked(char_index);
            }
            if *to_match == '\n' {
                return false;
            } else {
                for child in children {
                    if recursive_pure_match(node_vec, chars, child, char_index + 1) {
                        return true;
                    }
                }
                return false;
            }
        }
        Node::EndOfLine { children, .. } => {
            if char_index == chars.len() {
                for child in children {
                    if recursive_pure_match(node_vec, chars, child, char_index) {
                        return true;
                    }
                }
                return false;
            }
            let to_match: &char;
            unsafe {
                to_match = chars.get_unchecked(char_index);
            }
            if *to_match == '\n' {
                {
                    for child in children {
                        if recursive_pure_match(node_vec, chars, child, char_index + 1) {
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
                    if recursive_pure_match(node_vec, chars, child, char_index) {
                        return true;
                    }
                }
                return false;
            }
            let to_match: &char;
            unsafe {
                to_match = chars.get_unchecked(char_index);
            }
            if *to_match == '\n' {
                {
                    for child in children {
                        if recursive_pure_match(node_vec, chars, child, char_index + 1) {
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
            let to_match: &char;
            unsafe {
                to_match = chars.get_unchecked(char_index);
            }
            if to_match == character {
                for child in children {
                    if recursive_pure_match(node_vec, chars, child, char_index + 1) {
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
            let to_match: &char;
            unsafe {
                to_match = chars.get_unchecked(char_index);
            }
            if to_match != character {
                for child in children {
                    if recursive_pure_match(node_vec, chars, child, char_index + 1) {
                        return true;
                    }
                }
                return false;
            }
            return false;
        }
    }
}

pub(crate) fn get_index_match(node_vec: &[Node], chars: &[char], node_index: &usize, char_index: usize) -> Option<usize> {
    let node: &Node;
    unsafe {
        node = node_vec.get_unchecked(*node_index);
    };
    match node {
        Node::Transition { children, .. } => {
            for child in children {
                let m = get_index_match(node_vec, chars, child, char_index);
                match m {
                    Some(_) => return m,
                    None => (),
                }
            }
            return None;
        }
        Node::Inclusive { characters, children, .. } => {
            if char_index == chars.len() {
                return None;
            }
            let to_match: &char;
            unsafe {
                to_match = chars.get_unchecked(char_index);
            }
            if characters.contains(to_match) {
                for child in children {
                    let m = get_index_match(node_vec, chars, child, char_index + 1);
                    match m {
                        Some(_) => return m,
                        None => (),
                    }
                }
                return None;
            } else {
                return None;
            }
        }
        Node::End => {
            return Some(char_index);
        }
        Node::Exclusive { characters, children, .. } => {
            if char_index == chars.len() {
                return None;
            }
            let to_match: &char;
            unsafe {
                to_match = chars.get_unchecked(char_index);
            }
            if characters.contains(to_match) {
                return None;
            } else {
                for child in children {
                    let m = get_index_match(node_vec, chars, child, char_index + 1);
                    match m {
                        Some(_) => return m,
                        None => (),
                    }
                }
                return None;
            }
        }
        Node::MatchAll { children, .. } => {
            if char_index == chars.len() {
                return None;
            }
            let to_match: &char;
            unsafe {
                to_match = chars.get_unchecked(char_index);
            }
            if *to_match == '\n' {
                return None;
            } else {
                for child in children {
                    let m = get_index_match(node_vec, chars, child, char_index + 1);
                    match m {
                        Some(_) => return m,
                        None => (),
                    }
                }
                return None;
            }
        }
        Node::EndOfLine { children, .. } => {
            if char_index == chars.len() {
                for child in children {
                    let m = get_index_match(node_vec, chars, child, char_index);
                    match m {
                        Some(_) => return m,
                        None => (),
                    }
                }
                return None;
            }
            let to_match: &char;
            unsafe {
                to_match = chars.get_unchecked(char_index);
            }
            if *to_match == '\n' {
                {
                    for child in children {
                        let m = get_index_match(node_vec, chars, child, char_index + 1);
                        match m {
                            Some(_) => return m,
                            None => (),
                        }
                    }
                    return None;
                }
            }
            return None;
        }
        Node::BeginningOfLine { children, .. } => {
            if char_index == 0 {
                for child in children {
                    let m = get_index_match(node_vec, chars, child, char_index);
                    match m {
                        Some(_) => return m,
                        None => (),
                    }
                }
                return None;
            }
            let to_match: &char;
            unsafe {
                to_match = chars.get_unchecked(char_index);
            }
            if *to_match == '\n' {
                {
                    for child in children {
                        let m = get_index_match(node_vec, chars, child, char_index + 1);
                        match m {
                            Some(_) => return m,
                            None => (),
                        }
                    }
                    return None;
                }
            }
            return None;
        }
        Node::MatchOne { character, children } => {
            if char_index == chars.len() {
                return None;
            }
            let to_match: &char;
            unsafe {
                to_match = chars.get_unchecked(char_index);
            }
            if to_match == character {
                for child in children {
                    let m = get_index_match(node_vec, chars, child, char_index + 1);
                    match m {
                        Some(_) => return m,
                        None => (),
                    }
                }
                return None;
            }
            return None;
        }
        Node::NotMatchOne { character, children } => {
            if char_index == chars.len() {
                return None;
            }
            let to_match: &char;
            unsafe {
                to_match = chars.get_unchecked(char_index);
            }
            if to_match != character {
                for child in children {
                    let m = get_index_match(node_vec, chars, child, char_index + 1);
                    match m {
                        Some(_) => return m,
                        None => (),
                    }
                }
                return None;
            }
            return None;
        }
    }
}
