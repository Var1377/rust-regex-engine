use super::constants::W;
use std::collections::VecDeque;

// struct Capture {
//     string: Vec<char>,
//     children: Vec<Capture>,
// }

pub type CapturesMap = fxhash::FxHashMap::<u32, Vec<(usize, usize)>>;

use super::compiled_node::{CompiledNode, CNode::*, *};

use crate::utf_8::*;

pub(crate) fn backtrack_match_indices(nodes: &[CompiledNode], string: &[u8], start_node: usize, callstack: &mut Vec<(usize, usize, usize)>) -> Vec<(usize, usize)> {

    let mut node_index = start_node;
    let mut string_index = 0;
    let mut start_string_index = 0;
    // Node index, string index, child
    let mut recursion_stack = Vec::new();
    let mut completed_recursion_stack = Vec::new();
    // String Index, Node Index
    let mut lookahead_stack = Vec::<(usize, usize)>::new();

    let mut out = Vec::new();
    'outer: loop {
        let node = unsafe {nodes.get_unchecked(node_index)};
        let string_data = decode_utf8(&string[string_index..]);
        match &node.node {
            Match(match_node) => {
                match string_data {
                    Some((c, len)) => {
                        if match_node.is_match(&c) {
                            match &node.children {
                                Children::Multiple(vec) => {
                                    callstack.push((node_index, string_index + len, 1));
                                    node_index = *unsafe{vec.get_unchecked(0)};
                                },
                                Children::Single(num) => {
                                    node_index = *num;
                                },
                                Children::None => panic!("Match node has no children"), 
                            }
                            string_index += len;
                            continue 'outer
                        }
                    },
                    None => (),
                }
            }
            Anchor(anchor_node) => {
                if anchor_node.is_match(string_index, string, string_data) {
                    match &node.children {
                        Children::Multiple(vec) => {
                            callstack.push((node_index, string_index, 1));
                            node_index = *unsafe{vec.get_unchecked(0)};
                        },
                        Children::Single(num) => {
                            node_index = *num;
                        },
                        Children::None => panic!("Anchor node has no children"), 
                    }
                    continue 'outer
                }
            }
            Special(special_node) => {
                use SpecialNode::*;
                match special_node {
                    DropStack => callstack.clear(),
                    GlobalRecursion => {
                        recursion_stack.push(node_index);
                        callstack.push((node_index, string_index, 0));
                        node_index = start_node;
                        continue 'outer;
                    }
                    StartLookAhead => {
                        lookahead_stack.push((string_index, node_index));
                    }
                    EndLookAhead => {
                        let res = lookahead_stack.pop().unwrap();
                        while let Some((node_idx, _str_idx, _child)) = callstack.pop() {
                            // println!("Removed {:?}", nodes.get(node_idx));
                            if node_idx == res.1 {
                                break;
                            }
                        }
                        string_index = res.0;
                        callstack.push((node_index, res.0, 0));
                    }
                    StartNegativeLookAhead => {
                        lookahead_stack.push((string_index, node_index));
                        match &node.children {
                            Children::Multiple(vec) => {
                                callstack.push((node_index, string_index, 1));
                                // node_index = *unsafe{vec.get_unchecked(0)};
                                node_index = *vec.get(0).unwrap();
                            },
                            _ => panic!("Lookaround nodes should be Children::Multiple")
                        }
                        continue 'outer
                    }
                    EndNegativeLookAhead => {
                        let (_s, n) = lookahead_stack.pop().unwrap();
                        while let Some((node_idx, _str_idx, _child)) = callstack.pop() {
                            if node_idx == n {
                                break
                            }
                        }
                    }
                    _ => unimplemented!(),
                }
                match &node.children {
                    Children::Multiple(vec) => {
                        callstack.push((node_index, string_index, 1));
                        node_index = *unsafe{vec.get_unchecked(0)};
                    },
                    Children::Single(num) => {
                        node_index = *num;
                    },
                    Children::None => panic!("Behaviour node has no children"), 
                }
                continue 'outer
            },
            Behaviour(_) => {
                match &node.children {
                    Children::Multiple(vec) => {
                        callstack.push((node_index, string_index, 1));
                        node_index = *unsafe{vec.get_unchecked(0)};
                    },
                    Children::Single(num) => {
                        node_index = *num;
                    },
                    Children::None => panic!("Behaviour node has no children"), 
                }
                continue 'outer
            }
            End => {
                match recursion_stack.pop() {
                    Some(n) => {
                        completed_recursion_stack.push(n);
                        callstack.push((node_index, string_index, 0));
                        callstack.push((n, string_index, 0));
                    }
                    None => {
                        out.push((start_string_index, string_index));
                        start_string_index = string_index;
                        callstack.clear();
                    }
                }
            },
        }
        '_inner : loop {
            // println!("Backtracking {:?}, {:?}", recurstion_stack, completed_recurstion_stack);
            match callstack.pop() {
                Some((node_idx, string_idx, mut child)) => {
                    string_index = string_idx;
                    let node = unsafe {nodes.get_unchecked(node_idx)};
                    match &node.node {
                        End => {
                            recursion_stack.push(completed_recursion_stack.pop().unwrap());
                        }
                        Special(special) => {
                            use SpecialNode::*;
                            match special {
                                GlobalRecursion => {
                                    recursion_stack.pop();
                                }
                                StartLookAhead => {
                                    match &node.children {
                                        Children::Multiple(vec) => {
                                            match vec.get(child) {
                                                Some(new_child) => {
                                                    child += 1;
                                                    node_index = *new_child;
                                                    callstack.push((node_idx, string_idx, child));
                                                    // continue 'outer;
                                                }
                                                None => {lookahead_stack.pop();},
                                            }
                                        },
                                        _ => panic!("Lookaround nodes should be Children::Multiple")
                                    }
                                    continue 'outer
                                }
                                StartNegativeLookAhead => {
                                    match &node.children {
                                        Children::Multiple(vec) => {
                                            match vec.get(child) {
                                                Some(new_child) => {
                                                    child += 1;
                                                    node_index = *new_child;
                                                    callstack.push((node_idx, string_idx, child));
                                                    // continue 'outer;
                                                }
                                                None => {
                                                    let popped = lookahead_stack.pop().unwrap();
                                                    callstack.push((node_idx + 1, popped.0, 0))
                                                },
                                            }
                                            continue 'outer
                                        },
                                        _ => panic!("Lookaround nodes should be Children::Multiple")
                                    }
                                }
                                _ => (),
                            }
                        },
                        End => {
                            recursion_stack.push(completed_recursion_stack.pop().unwrap());
                        }
                        _ => (),
                    }
                    match &node.children {
                        Children::Multiple(vec) => {
                            match vec.get(child) {
                                Some(new_child) => {
                                    child += 1;
                                    node_index = *new_child;
                                    callstack.push((node_idx, string_idx, child));
                                    continue 'outer;
                                }
                                None => (),
                            }
                        },
                        Children::Single(c) => {
                            node_index = *c;
                            continue 'outer;
                        }
                        _ => ()
                    }
                },
                None => {
                    if start_string_index < string.len() {
                        start_string_index = next_utf8(string, start_string_index);
                        string_index = start_string_index;
                        node_index = start_node;
                        continue 'outer
                    } else {
                        return out;
                    }
                }
            }
        }
    }
}

pub(crate) fn backtrack_first_match(nodes: &[CompiledNode], string: &[u8], start_node: usize, callstack: &mut Vec<(usize, usize, usize)>) -> Option<(usize, usize)> {

    let mut node_index = start_node;
    let mut string_index = 0;
    let mut start_string_index = 0;
    // Node index, string index, child
    let mut recursion_stack = Vec::new();
    let mut completed_recursion_stack = Vec::new();
    // String Index, Node Index
    let mut lookahead_stack = Vec::<(usize, usize)>::new();

    'outer: loop {
        let node = unsafe {nodes.get_unchecked(node_index)};
        let string_data = decode_utf8(&string[string_index..]);
        match &node.node {
            Match(match_node) => {
                match string_data {
                    Some((c, len)) => {
                        if match_node.is_match(&c) {
                            match &node.children {
                                Children::Multiple(vec) => {
                                    callstack.push((node_index, string_index + len, 1));
                                    node_index = *unsafe{vec.get_unchecked(0)};
                                },
                                Children::Single(num) => {
                                    node_index = *num;
                                },
                                Children::None => panic!("Match node has no children"), 
                            }
                            string_index += len;
                            continue 'outer
                        }
                    },
                    None => (),
                }
            }
            Anchor(anchor_node) => {
                if anchor_node.is_match(string_index, string, string_data) {
                    match &node.children {
                        Children::Multiple(vec) => {
                            callstack.push((node_index, string_index, 1));
                            node_index = *unsafe{vec.get_unchecked(0)};
                        },
                        Children::Single(num) => {
                            node_index = *num;
                        },
                        Children::None => panic!("Anchor node has no children"), 
                    }
                    continue 'outer
                }
            }
            Special(special_node) => {
                use SpecialNode::*;
                match special_node {
                    DropStack => callstack.clear(),
                    GlobalRecursion => {
                        recursion_stack.push(node_index);
                        callstack.push((node_index, string_index, 0));
                        node_index = start_node;
                        continue 'outer;
                    }
                    StartLookAhead => {
                        lookahead_stack.push((string_index, node_index));
                    }
                    EndLookAhead => {
                        let res = lookahead_stack.pop().unwrap();
                        while let Some((node_idx, _str_idx, _child)) = callstack.pop() {
                            // println!("Removed {:?}", nodes.get(node_idx));
                            if node_idx == res.1 {
                                break;
                            }
                        }
                        string_index = res.0;
                        callstack.push((node_index, res.0, 0));
                    }
                    StartNegativeLookAhead => {
                        lookahead_stack.push((string_index, node_index));
                        match &node.children {
                            Children::Multiple(vec) => {
                                callstack.push((node_index, string_index, 1));
                                // node_index = *unsafe{vec.get_unchecked(0)};
                                node_index = *vec.get(0).unwrap();
                            },
                            _ => panic!("Lookaround nodes should be Children::Multiple")
                        }
                        continue 'outer
                    }
                    EndNegativeLookAhead => {
                        let (_s, n) = lookahead_stack.pop().unwrap();
                        while let Some((node_idx, _str_idx, _child)) = callstack.pop() {
                            if node_idx == n {
                                break
                            }
                        }
                    }
                    _ => unimplemented!(),
                }
                match &node.children {
                    Children::Multiple(vec) => {
                        callstack.push((node_index, string_index, 1));
                        node_index = *unsafe{vec.get_unchecked(0)};
                    },
                    Children::Single(num) => {
                        node_index = *num;
                    },
                    Children::None => panic!("Behaviour node has no children"), 
                }
                continue 'outer
            },
            Behaviour(_) => {
                match &node.children {
                    Children::Multiple(vec) => {
                        callstack.push((node_index, string_index, 1));
                        node_index = *unsafe{vec.get_unchecked(0)};
                    },
                    Children::Single(num) => {
                        node_index = *num;
                    },
                    Children::None => panic!("Behaviour node has no children"), 
                }
                continue 'outer
            }
            End => {
                match recursion_stack.pop() {
                    Some(n) => {
                        completed_recursion_stack.push(n);
                        callstack.push((node_index, string_index, 0));
                        callstack.push((n, string_index, 0));
                    }
                    None => {
                        callstack.clear();
                        return Some((start_string_index, string_index));
                    }
                }
            },
        }
        '_inner : loop {
            // println!("Backtracking {:?}, {:?}", recurstion_stack, completed_recurstion_stack);
            match callstack.pop() {
                Some((node_idx, string_idx, mut child)) => {
                    string_index = string_idx;
                    let node = unsafe {nodes.get_unchecked(node_idx)};
                    match &node.node {
                        End => {
                            recursion_stack.push(completed_recursion_stack.pop().unwrap());
                        }
                        Special(special) => {
                            use SpecialNode::*;
                            match special {
                                GlobalRecursion => {
                                    recursion_stack.pop();
                                }
                                StartLookAhead => {
                                    match &node.children {
                                        Children::Multiple(vec) => {
                                            match vec.get(child) {
                                                Some(new_child) => {
                                                    child += 1;
                                                    node_index = *new_child;
                                                    callstack.push((node_idx, string_idx, child));
                                                    // continue 'outer;
                                                }
                                                None => {lookahead_stack.pop();},
                                            }
                                        },
                                        _ => panic!("Lookaround nodes should be Children::Multiple")
                                    }
                                    continue 'outer
                                }
                                StartNegativeLookAhead => {
                                    match &node.children {
                                        Children::Multiple(vec) => {
                                            match vec.get(child) {
                                                Some(new_child) => {
                                                    child += 1;
                                                    node_index = *new_child;
                                                    callstack.push((node_idx, string_idx, child));
                                                    // continue 'outer;
                                                }
                                                None => {
                                                    let popped = lookahead_stack.pop().unwrap();
                                                    callstack.push((node_idx + 1, popped.0, 0))
                                                },
                                            }
                                            continue 'outer
                                        },
                                        _ => panic!("Lookaround nodes should be Children::Multiple")
                                    }
                                }
                                _ => (),
                            }
                        },
                        End => {
                            recursion_stack.push(completed_recursion_stack.pop().unwrap());
                        }
                        _ => (),
                    }
                    match &node.children {
                        Children::Multiple(vec) => {
                            match vec.get(child) {
                                Some(new_child) => {
                                    child += 1;
                                    node_index = *new_child;
                                    callstack.push((node_idx, string_idx, child));
                                    continue 'outer;
                                }
                                None => (),
                            }
                        },
                        Children::Single(c) => {
                            node_index = *c;
                            continue 'outer;
                        }
                        _ => ()
                    }
                },
                None => {
                    if start_string_index < string.len() {
                        start_string_index = next_utf8(string, start_string_index);
                        string_index = start_string_index;
                        node_index = start_node;
                        continue 'outer
                    } else {
                        return None;
                    }
                }
            }
        }
    }
}


pub(crate) fn backtrack_pure_match(nodes: &[CompiledNode], string: &[u8], start_node: usize, callstack: &mut Vec<(usize, usize, usize)>) -> bool {

    let mut node_index = start_node;
    let mut string_index = 0;
    let mut start_string_index = 0;
    // Node index, string index, child
    let mut recursion_stack = Vec::new();
    let mut completed_recursion_stack = Vec::new();
    // String Index, Node Index
    let mut lookahead_stack = Vec::<(usize, usize)>::new();

    'outer: loop {
        let node = unsafe {nodes.get_unchecked(node_index)};
        let string_data = decode_utf8(&string[string_index..]);
        match &node.node {
            Match(match_node) => {
                match string_data {
                    Some((c, len)) => {
                        if match_node.is_match(&c) {
                            match &node.children {
                                Children::Multiple(vec) => {
                                    callstack.push((node_index, string_index + len, 1));
                                    node_index = *unsafe{vec.get_unchecked(0)};
                                },
                                Children::Single(num) => {
                                    node_index = *num;
                                },
                                Children::None => panic!("Match node has no children"), 
                            }
                            string_index += len;
                            continue 'outer
                        }
                    },
                    None => (),
                }
            }
            Anchor(anchor_node) => {
                if anchor_node.is_match(string_index, string, string_data) {
                    match &node.children {
                        Children::Multiple(vec) => {
                            callstack.push((node_index, string_index, 1));
                            node_index = *unsafe{vec.get_unchecked(0)};
                        },
                        Children::Single(num) => {
                            node_index = *num;
                        },
                        Children::None => panic!("Anchor node has no children"), 
                    }
                    continue 'outer
                }
            }
            Special(special_node) => {
                use SpecialNode::*;
                match special_node {
                    DropStack => callstack.clear(),
                    GlobalRecursion => {
                        recursion_stack.push(node_index);
                        callstack.push((node_index, string_index, 0));
                        node_index = start_node;
                        continue 'outer;
                    }
                    StartLookAhead => {
                        lookahead_stack.push((string_index, node_index));
                    }
                    EndLookAhead => {
                        let res = lookahead_stack.pop().unwrap();
                        while let Some((node_idx, _str_idx, _child)) = callstack.pop() {
                            // println!("Removed {:?}", nodes.get(node_idx));
                            if node_idx == res.1 {
                                break;
                            }
                        }
                        string_index = res.0;
                        callstack.push((node_index, res.0, 0));
                    }
                    StartNegativeLookAhead => {
                        lookahead_stack.push((string_index, node_index));
                        match &node.children {
                            Children::Multiple(vec) => {
                                callstack.push((node_index, string_index, 1));
                                // node_index = *unsafe{vec.get_unchecked(0)};
                                node_index = *vec.get(0).unwrap();
                            },
                            _ => panic!("Lookaround nodes should be Children::Multiple")
                        }
                        continue 'outer
                    }
                    EndNegativeLookAhead => {
                        let (_s, n) = lookahead_stack.pop().unwrap();
                        while let Some((node_idx, _str_idx, _child)) = callstack.pop() {
                            if node_idx == n {
                                break
                            }
                        }
                    }
                    _ => unimplemented!(),
                }
                match &node.children {
                    Children::Multiple(vec) => {
                        callstack.push((node_index, string_index, 1));
                        node_index = *unsafe{vec.get_unchecked(0)};
                    },
                    Children::Single(num) => {
                        node_index = *num;
                    },
                    Children::None => panic!("Behaviour node has no children"), 
                }
                continue 'outer
            },
            Behaviour(_) => {
                match &node.children {
                    Children::Multiple(vec) => {
                        callstack.push((node_index, string_index, 1));
                        node_index = *unsafe{vec.get_unchecked(0)};
                    },
                    Children::Single(num) => {
                        node_index = *num;
                    },
                    Children::None => panic!("Behaviour node has no children"), 
                }
                continue 'outer
            }
            End => {
                match recursion_stack.pop() {
                    Some(n) => {
                        completed_recursion_stack.push(n);
                        callstack.push((node_index, string_index, 0));
                        callstack.push((n, string_index, 0));
                    }
                    None => {
                        callstack.clear();
                        return true
                    }
                }
            },
        }
        '_inner : loop {
            // println!("Backtracking {:?}, {:?}", recurstion_stack, completed_recurstion_stack);
            match callstack.pop() {
                Some((node_idx, string_idx, mut child)) => {
                    string_index = string_idx;
                    let node = unsafe {nodes.get_unchecked(node_idx)};
                    match &node.node {
                        End => {
                            recursion_stack.push(completed_recursion_stack.pop().unwrap());
                        }
                        Special(special) => {
                            use SpecialNode::*;
                            match special {
                                GlobalRecursion => {
                                    recursion_stack.pop();
                                }
                                StartLookAhead => {
                                    match &node.children {
                                        Children::Multiple(vec) => {
                                            match vec.get(child) {
                                                Some(new_child) => {
                                                    child += 1;
                                                    node_index = *new_child;
                                                    callstack.push((node_idx, string_idx, child));
                                                    // continue 'outer;
                                                }
                                                None => {lookahead_stack.pop();},
                                            }
                                        },
                                        _ => panic!("Lookaround nodes should be Children::Multiple")
                                    }
                                    continue 'outer
                                }
                                StartNegativeLookAhead => {
                                    match &node.children {
                                        Children::Multiple(vec) => {
                                            match vec.get(child) {
                                                Some(new_child) => {
                                                    child += 1;
                                                    node_index = *new_child;
                                                    callstack.push((node_idx, string_idx, child));
                                                    // continue 'outer;
                                                }
                                                None => {
                                                    let popped = lookahead_stack.pop().unwrap();
                                                    callstack.push((node_idx + 1, popped.0, 0))
                                                },
                                            }
                                            continue 'outer
                                        },
                                        _ => panic!("Lookaround nodes should be Children::Multiple")
                                    }
                                }
                                _ => (),
                            }
                        },
                        End => {
                            recursion_stack.push(completed_recursion_stack.pop().unwrap());
                        }
                        _ => (),
                    }
                    match &node.children {
                        Children::Multiple(vec) => {
                            match vec.get(child) {
                                Some(new_child) => {
                                    child += 1;
                                    node_index = *new_child;
                                    callstack.push((node_idx, string_idx, child));
                                    continue 'outer;
                                }
                                None => (),
                            }
                        },
                        Children::Single(c) => {
                            node_index = *c;
                            continue 'outer;
                        }
                        _ => ()
                    }
                },
                None => {
                    if start_string_index < string.len() {
                        start_string_index = next_utf8(string, start_string_index);
                        string_index = start_string_index;
                        node_index = start_node;
                        continue 'outer
                    } else {
                        return false;
                    }
                }
            }
        }
    }
}