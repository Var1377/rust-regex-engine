use super::nfa::*;

pub(crate) fn recursive_pure_match(node_vec: &[Node], chars: &[char], node_index: &usize, char_index: usize) -> bool {
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
