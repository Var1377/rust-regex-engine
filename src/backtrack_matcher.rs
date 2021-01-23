use super::constants::W;
use std::collections::VecDeque;
use std::hint::unreachable_unchecked;

// struct Capture {
//     string: Vec<char>,
//     children: Vec<Capture>,
// }

// pub type CapturesMap = fxhash::FxHashMap<u32, Vec<(usize, usize)>>;

use super::compiled_node::{CNode::*, CompiledNode, *};
use crate::root_node_optimizer::RootNode;
use crate::utf_8::*;
use BackTrackToken::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackTrackToken {
    // String index, child(ren)
    // Special case for a branch with only 2 directions, occurs with +, ?, *
    // Worth it for performance reasons
    Single(usize, usize),
    Multiple(usize, usize, usize),
    // For recursion
    End,
    // For positive and negatiue lookarounds
    PopAltStack,
    NegativeLookahead(usize, usize),
    // For atomic groups
    Atomic,
}

pub(crate) fn backtrack_match_indices(
    nodes: &[CompiledNode],
    string: &[u8],
    start_node: usize,
    // Node index, string index, child
    callstack: &mut Vec<BackTrackToken>,
    root_node: &Option<RootNode>,
) -> Vec<(usize, usize)> {
    let mut out = Vec::new();

    let mut node_index = start_node;
    let mut string_index = 0;
    let mut start_string_index = 0;
    // Node index, string index, child
    let mut completed_recursion_stack = Vec::new();
    // String Index, Node Index
    let mut alt_stack = Vec::<(usize, usize)>::new();

    'outer: loop {
        let node = unsafe { nodes.get_unchecked(node_index) };
        let string_data = decode_utf8(&string[string_index..]);
        match &node.node {
            Match(match_node) => match string_data {
                Some((c, len)) => {
                    if match_node.is_match(&c) {
                        match &node.children {
                            Children::Multiple(vec) => {
                                if vec.len() == 2 {
                                    callstack.push(BackTrackToken::Single(string_index + len, unsafe { *vec.get_unchecked(1) }));
                                } else {
                                    callstack.push(BackTrackToken::Multiple(string_index + len, node_index, 1));
                                }
                                node_index = *unsafe { vec.get_unchecked(0) };
                            }
                            Children::Single(num) => {
                                node_index = *num;
                            }
                            Children::None => panic!("Match node has no children"),
                        }
                        string_index += len;
                        continue 'outer;
                    }
                }
                None => (),
            },
            Anchor(anchor_node) => {
                if anchor_node.is_match(string_index, string, string_data) {
                    match &node.children {
                        Children::Multiple(vec) => {
                            if vec.len() == 2 {
                                callstack.push(BackTrackToken::Single(string_index, unsafe { *vec.get_unchecked(1) }));
                            } else {
                                callstack.push(BackTrackToken::Multiple(string_index, node_index, 1));
                            }
                            node_index = *unsafe { vec.get_unchecked(0) };
                        }
                        Children::Single(num) => {
                            node_index = *num;
                        }
                        Children::None => panic!("Anchor node has no children"),
                    }
                    continue 'outer;
                }
            }
            Special(special_node) => {
                use SpecialNode::*;
                match special_node {
                    DropStack => callstack.clear(),
                    GlobalRecursion => {
                        alt_stack.push((string_index, node_index));
                        callstack.push(PopAltStack);
                        node_index = start_node;
                        continue 'outer;
                    }
                    StartLookAhead => {
                        alt_stack.push((string_index, node_index));
                        callstack.push(PopAltStack);
                    }
                    EndLookAhead => {
                        let res = alt_stack.pop().unwrap();
                        while let Some(token) = callstack.pop() {
                            // println!("Removed {:?}", nodes.get(node_idx));
                            if token == PopAltStack {
                                break;
                            }
                        }
                        string_index = res.0;
                    }
                    StartNegativeLookAhead => {
                        // alt_stack.push((string_index, node_index + 1));
                        callstack.push(NegativeLookahead(string_index, node_index + 1));
                    }
                    EndNegativeLookAhead => {
                        // let (old_string_idx, _old_node_idx,) = alt_stack.pop().unwrap();
                        while let Some(token) = callstack.pop() {
                            // println!("Removed {:?}", nodes.get(node_idx));
                            if let NegativeLookahead(_, _) = token {
                                break;
                            }
                        }
                        // string_index = old_string_idx;
                    }
                    StartAtomic => {
                        callstack.push(Atomic);
                    }
                    EndAtomic => {
                        while let Some(token) = callstack.pop() {
                            if token == Atomic {
                                break;
                            }
                        }
                    }
                    _ => unimplemented!(),
                }
                match &node.children {
                    Children::Multiple(vec) => {
                        if vec.len() == 2 {
                            callstack.push(BackTrackToken::Single(string_index, unsafe { *vec.get_unchecked(1) }));
                        } else {
                            callstack.push(BackTrackToken::Multiple(string_index, node_index, 1));
                        }
                        node_index = *unsafe { vec.get_unchecked(0) };
                    }
                    Children::Single(num) => {
                        node_index = *num;
                    }
                    Children::None => panic!("Anchor node has no children"),
                }
                continue 'outer;
            }
            Behaviour(_) => {
                match &node.children {
                    Children::Multiple(vec) => {
                        if vec.len() == 2 {
                            callstack.push(BackTrackToken::Single(string_index, unsafe { *vec.get_unchecked(1) }));
                        } else {
                            callstack.push(BackTrackToken::Multiple(string_index, node_index, 1));
                        }
                        node_index = *unsafe { vec.get_unchecked(0) };
                    }
                    Children::Single(num) => {
                        node_index = *num;
                    }
                    Children::None => panic!("Anchor node has no children"),
                }
                continue 'outer;
            }
            CNode::End => match alt_stack.pop() {
                Some(n) => {
                    completed_recursion_stack.push(n.1);
                    node_index = n.1;
                    callstack.push(BackTrackToken::End);
                    continue 'outer;
                }
                None => {
                    callstack.clear();
                    out.push((start_string_index, string_index));
                    start_string_index = string_index;
                }
            },
        }
        loop {
            match callstack.pop() {
                Some(token) => match token {
                    Single(str_idx, node_idx) => {
                        node_index = node_idx;
                        string_index = str_idx;
                        continue 'outer;
                    }
                    Multiple(str_idx, node_idx, child) => {
                        string_index = str_idx;
                        let node = unsafe { nodes.get_unchecked(node_idx) };
                        match &node.children {
                            Children::Multiple(vec) => {
                                let new_child = unsafe { *vec.get_unchecked(child) };
                                if child + 2 == vec.len() {
                                    callstack.push(Single(str_idx, *unsafe { vec.get_unchecked(child + 1) }));
                                } else {
                                    callstack.push(Multiple(str_idx, node_idx, child + 1));
                                }
                                node_index = new_child;
                            }
                            _ => unreachable!(),
                        }
                        continue 'outer;
                    }
                    PopAltStack => {
                        alt_stack.pop();
                    }
                    NegativeLookahead(string_idx, node_idx) => {
                        let node = nodes.get(node_idx).unwrap();
                        string_index = string_idx;
                        match &node.children {
                            Children::Multiple(vec) => {
                                if vec.len() == 2 {
                                    callstack.push(BackTrackToken::Single(string_index, unsafe { *vec.get_unchecked(1) }));
                                } else {
                                    callstack.push(BackTrackToken::Multiple(string_index, node_index, 1));
                                }
                                node_index = *unsafe { vec.get_unchecked(0) };
                            }
                            Children::Single(num) => {
                                node_index = *num;
                            }
                            Children::None => panic!("Anchor node has no children"),
                        }
                        continue 'outer;
                    }
                    _ => (),
                },
                None => {
                    start_string_index = next_utf8(string, start_string_index);
                    if let Some(root_node) = root_node {
                        match root_node.run(string, start_string_index) {
                            Some(idx) => {
                                node_index = root_node.child;
                                start_string_index = idx;
                                string_index = idx;
                                continue 'outer;
                            }
                            None => return out,
                        }
                    } else if start_string_index < string.len() {
                        string_index = start_string_index;
                        node_index = start_node;
                        continue 'outer;
                    } else {
                        return out;
                    }
                }
            }
        }
    }
}

pub(crate) fn backtrack_pure_match(
    nodes: &[CompiledNode],
    string: &[u8],
    start_node: usize,
    callstack: &mut Vec<BackTrackToken>,
    root_node: &Option<RootNode>,
) -> bool {
    let mut node_index = start_node;
    let mut string_index = 0;
    let mut start_string_index = 0;
    // Node index, string index, child
    let mut completed_recursion_stack = Vec::new();
    // String Index, Node Index
    let mut alt_stack = Vec::<(usize, usize)>::new();

    'outer: loop {
        let node = unsafe { nodes.get_unchecked(node_index) };
        let string_data = decode_utf8(&string[string_index..]);
        match &node.node {
            Match(match_node) => match string_data {
                Some((c, len)) => {
                    if match_node.is_match(&c) {
                        match &node.children {
                            Children::Multiple(vec) => {
                                if vec.len() == 2 {
                                    callstack.push(BackTrackToken::Single(string_index + len, unsafe { *vec.get_unchecked(1) }));
                                } else {
                                    callstack.push(BackTrackToken::Multiple(string_index + len, node_index, 1));
                                }
                                node_index = *unsafe { vec.get_unchecked(0) };
                            }
                            Children::Single(num) => {
                                node_index = *num;
                            }
                            Children::None => panic!("Match node has no children"),
                        }
                        string_index += len;
                        continue 'outer;
                    }
                }
                None => (),
            },
            Anchor(anchor_node) => {
                if anchor_node.is_match(string_index, string, string_data) {
                    match &node.children {
                        Children::Multiple(vec) => {
                            if vec.len() == 2 {
                                callstack.push(BackTrackToken::Single(string_index, unsafe { *vec.get_unchecked(1) }));
                            } else {
                                callstack.push(BackTrackToken::Multiple(string_index, node_index, 1));
                            }
                            node_index = *unsafe { vec.get_unchecked(0) };
                        }
                        Children::Single(num) => {
                            node_index = *num;
                        }
                        Children::None => panic!("Anchor node has no children"),
                    }
                    continue 'outer;
                }
            }
            Special(special_node) => {
                use SpecialNode::*;
                match special_node {
                    DropStack => callstack.clear(),
                    GlobalRecursion => {
                        alt_stack.push((string_index, node_index));
                        callstack.push(PopAltStack);
                        node_index = start_node;
                        continue 'outer;
                    }
                    StartLookAhead => {
                        alt_stack.push((string_index, node_index));
                        callstack.push(PopAltStack);
                    }
                    EndLookAhead => {
                        let res = alt_stack.pop().unwrap();
                        while let Some(token) = callstack.pop() {
                            // println!("Removed {:?}", nodes.get(node_idx));
                            if token == PopAltStack {
                                break;
                            }
                        }
                        string_index = res.0;
                    }
                    StartNegativeLookAhead => {
                        // alt_stack.push((string_index, node_index + 1));
                        callstack.push(NegativeLookahead(string_index, node_index + 1));
                    }
                    EndNegativeLookAhead => {
                        // let (old_string_idx, _old_node_idx,) = alt_stack.pop().unwrap();
                        while let Some(token) = callstack.pop() {
                            // println!("Removed {:?}", nodes.get(node_idx));
                            if let NegativeLookahead(_, _) = token {
                                break;
                            }
                        }
                        // string_index = old_string_idx;
                    }
                    StartAtomic => {
                        callstack.push(Atomic);
                    }
                    EndAtomic => {
                        while let Some(token) = callstack.pop() {
                            if token == Atomic {
                                break;
                            }
                        }
                    }
                    _ => unimplemented!(),
                }
                match &node.children {
                    Children::Multiple(vec) => {
                        if vec.len() == 2 {
                            callstack.push(BackTrackToken::Single(string_index, unsafe { *vec.get_unchecked(1) }));
                        } else {
                            callstack.push(BackTrackToken::Multiple(string_index, node_index, 1));
                        }
                        node_index = *unsafe { vec.get_unchecked(0) };
                    }
                    Children::Single(num) => {
                        node_index = *num;
                    }
                    Children::None => panic!("Anchor node has no children"),
                }
                continue 'outer;
            }
            Behaviour(_) => {
                match &node.children {
                    Children::Multiple(vec) => {
                        if vec.len() == 2 {
                            callstack.push(BackTrackToken::Single(string_index, unsafe { *vec.get_unchecked(1) }));
                        } else {
                            callstack.push(BackTrackToken::Multiple(string_index, node_index, 1));
                        }
                        node_index = *unsafe { vec.get_unchecked(0) };
                    }
                    Children::Single(num) => {
                        node_index = *num;
                    }
                    Children::None => panic!("Anchor node has no children"),
                }
                continue 'outer;
            }
            CNode::End => match alt_stack.pop() {
                Some(n) => {
                    completed_recursion_stack.push(n.1);
                    node_index = n.1;
                    callstack.push(BackTrackToken::End);
                    continue 'outer;
                }
                None => {
                    callstack.clear();
                    return true;
                }
            },
        }
        loop {
            match callstack.pop() {
                Some(token) => match token {
                    Single(str_idx, node_idx) => {
                        node_index = node_idx;
                        string_index = str_idx;
                        continue 'outer;
                    }
                    Multiple(str_idx, node_idx, child) => {
                        string_index = str_idx;
                        let node = unsafe { nodes.get_unchecked(node_idx) };
                        match &node.children {
                            Children::Multiple(vec) => {
                                let new_child = unsafe { *vec.get_unchecked(child) };
                                if child + 2 == vec.len() {
                                    callstack.push(Single(str_idx, *unsafe { vec.get_unchecked(child + 1) }));
                                } else {
                                    callstack.push(Multiple(str_idx, node_idx, child + 1));
                                }
                                node_index = new_child;
                            }
                            _ => unreachable!(),
                        }
                        continue 'outer;
                    }
                    PopAltStack => {
                        alt_stack.pop();
                    }
                    NegativeLookahead(string_idx, node_idx) => {
                        let node = nodes.get(node_idx).unwrap();
                        string_index = string_idx;
                        match &node.children {
                            Children::Multiple(vec) => {
                                if vec.len() == 2 {
                                    callstack.push(BackTrackToken::Single(string_index, unsafe { *vec.get_unchecked(1) }));
                                } else {
                                    callstack.push(BackTrackToken::Multiple(string_index, node_index, 1));
                                }
                                node_index = *unsafe { vec.get_unchecked(0) };
                            }
                            Children::Single(num) => {
                                node_index = *num;
                            }
                            Children::None => panic!("Anchor node has no children"),
                        }
                        continue 'outer;
                    }
                    _ => (),
                },
                None => {
                    start_string_index = next_utf8(string, start_string_index);
                    if let Some(root_node) = root_node {
                        match root_node.run(string, start_string_index) {
                            Some(idx) => {
                                node_index = root_node.child;
                                start_string_index = idx;
                                string_index = idx;
                                continue 'outer;
                            }
                            None => return false,
                        }
                    } else if start_string_index < string.len() {
                        string_index = start_string_index;
                        node_index = start_node;
                        continue 'outer;
                    } else {
                        return false;
                    }
                }
            }
        }
    }
}
