use crate::compiled_node::*;
use crate::utf_8::*;
use fnv::{FnvHashSet, FnvBuildHasher};
use sorted_vec::*;
use derivative::Derivative;
use indexmap::IndexSet;
use std::collections::BTreeSet;

#[derive(Derivative)]
#[derivative(Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct NodeIndex(usize, #[derivative(Hash = "ignore", PartialEq = "ignore", PartialOrd = "ignore")] usize);

// much quicker than std::collections::BTreeSet for some unknown reason
type CustomHashSet = SortedSet<usize>;


pub(crate) fn pure_match(nodes: &[CompiledNode], string_bytes: &[u8], start_node_index: usize) -> bool {
    // println!("Breadth First Search engine invoked");
    let mut stack1 = CustomHashSet::default();
    stack1.insert(start_node_index);
    let mut stack2 = CustomHashSet::default();
    let mut stack_alt = true;
    let mut string_index = 0usize;
    let mut split_at = 0usize;
    // let mut offset = 0;
    '_outer: loop {
        let cached: Option<(char, usize)> = decode_utf8(&string_bytes[split_at..]);
        let (current_stack, to_add_stack) = if stack_alt {
            (&mut stack1, &mut stack2)
        } else {
            (&mut stack2, &mut stack1)
        };
        'inner: while let Some(node_idx) = current_stack.pop() {
            let node = unsafe { nodes.get_unchecked(node_idx) };
            // let node = nodes.get(node_idx).unwrap();
            // println!("{:?}", node);
            let success: bool;
            match &node.node {
                CNode::Match(match_node) => match cached {
                    Some(t) => {
                        let character = &t.0;
                        match match_node {
                            MatchNode::One(match_node) => match match_node {
                                One::MatchOne(c) => success = character == c,
                                One::NotMatchOne(c) => success = character != c,
                                One::MatchAll => success = true,
                            },
                            MatchNode::Range(match_node) => match match_node {
                                Range::Inclusive(chars) => success = chars.contains(character),
                                Range::Exclusive(chars) => success = !chars.contains(character),
                                Range::InclusiveRange(ranges) => {
                                    success = ranges.iter().find(|(start, end)| character >= start && character <= end).is_some()
                                }
                                Range::ExclusiveRange(ranges) => {
                                    success = ranges.iter().find(|(start, end)| character >= start && character <= end).is_none()
                                }
                            },
                        }
                        if success {
                            match &node.children {
                                Children::Multiple(vec) => vec.iter().for_each(|v| {to_add_stack.insert(*v);}),
                                Children::Single(s) => {to_add_stack.insert(*s);},
                                Children::None => panic!("Match node has no children"),
                            }
                        }
                    }
                    None => continue 'inner,
                },
                CNode::Anchor(anchor_node) => {
                    use crate::constants::*;
                    match anchor_node {
                        AnchorNode::StartOfString => success = split_at == 0,
                        AnchorNode::EndOfString => success = split_at >= string_bytes.len(),
                        AnchorNode::BeginningOfLine => success = split_at == 0 || string_bytes[split_at - 1] == b'\n',
                        AnchorNode::EndOfLine => success = split_at >= string_bytes.len() || string_bytes[split_at] == b'\n',
                        AnchorNode::WordBoundary => {
                            success = (split_at == 0
                                && match string_bytes.get(split_at).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                })
                                || (split_at == string_bytes.len()
                                    && match string_bytes.get(split_at - 1).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                        Some(b) => b,
                                        None => false,
                                    })
                                || (match string_bytes.get(split_at - 1).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                } && !(match string_bytes.get(split_at).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                }))
                                || (match string_bytes.get(split_at).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                } && !(match string_bytes.get(split_at - 1).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                }))
                        }
                        AnchorNode::NotWordBoundary => {
                            success = !((split_at == 0
                                && match string_bytes.get(split_at).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                })
                                || (split_at == string_bytes.len()
                                    && match string_bytes.get(split_at - 1).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                        Some(b) => b,
                                        None => false,
                                    })
                                || (match string_bytes.get(split_at - 1).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                } && !(match string_bytes.get(split_at).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                }))
                                || (match string_bytes.get(split_at).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                } && !(match string_bytes.get(split_at - 1).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                })))
                        }
                    }
                    if success {
                        match &node.children {
                            Children::Multiple(vec) => vec.iter().for_each(|v| {current_stack.insert(*v);}),
                            Children::Single(s) => {current_stack.insert(*s);},
                            Children::None => panic!("Anchor node has no children"),
                        }
                    }
                }
                CNode::Behaviour(_) => {
                    match &node.children {
                        Children::Multiple(vec) => vec.iter().for_each(|v| {current_stack.insert(*v);}),
                        Children::Single(s) => {current_stack.insert(*s);},
                        Children::None => panic!("Behaviour node has no children"),
                    }
                }
                CNode::Special(_) => panic!("Special Nodes not supported on the BFS engine"),
                CNode::End => return true,
            };
        }
        if to_add_stack.is_empty() {
            if string_index < string_bytes.len() - 1 {
                string_index = next_utf8(string_bytes, string_index);
                to_add_stack.insert(start_node_index);
                split_at = string_index;
            } else {
                return false;
            }
        }
        // println!("{:?}", to_add_stack);
        stack_alt = !stack_alt;

        if let Some((_, len)) = cached {
            split_at += len;
        }
    }
    // None
}

pub(crate) fn index_match(nodes: &[CompiledNode], string_bytes: &[u8], start_node_index: usize) -> Option<(usize, usize)> {
    // println!("Breadth First Search engine invoked");
    let mut stack1 = CustomHashSet::default();
    stack1.insert(start_node_index);
    let mut stack2 = CustomHashSet::default();
    let mut stack_alt = true;
    let mut string_index = 0usize;
    let mut split_at = 0usize;
    '_outer: loop {
        let cached: Option<(char, usize)> = decode_utf8(&string_bytes[split_at..]);
        let (current_stack, to_add_stack) = if stack_alt {
            (&mut stack1, &mut stack2)
        } else {
            (&mut stack2, &mut stack1)
        };
        'inner: while let Some(node_idx) = current_stack.pop() {
            let node = unsafe { nodes.get_unchecked(node_idx) };
            // let node = nodes.get(node_idx).unwrap();
            // println!("{:?}", node);
            let success: bool;
            match &node.node {
                CNode::Match(match_node) => match cached {
                    Some(t) => {
                        let character = &t.0;
                        match match_node {
                            MatchNode::One(match_node) => match match_node {
                                One::MatchOne(c) => success = character == c,
                                One::NotMatchOne(c) => success = character != c,
                                One::MatchAll => success = true,
                            },
                            MatchNode::Range(match_node) => match match_node {
                                Range::Inclusive(chars) => success = chars.contains(character),
                                Range::Exclusive(chars) => success = !chars.contains(character),
                                Range::InclusiveRange(ranges) => {
                                    success = ranges.iter().find(|(start, end)| character >= start && character <= end).is_some()
                                }
                                Range::ExclusiveRange(ranges) => {
                                    success = ranges.iter().find(|(start, end)| character >= start && character <= end).is_none()
                                }
                            },
                        }
                        if success {
                            match &node.children {
                                Children::Multiple(vec) => vec.iter().for_each(|v| {to_add_stack.insert(*v);}),
                                Children::Single(s) => {to_add_stack.insert(*s);},
                                Children::None => panic!("Match node has no children"),
                            }
                        }
                    }
                    None => continue 'inner,
                },
                CNode::Anchor(anchor_node) => {
                    use crate::constants::*;
                    match anchor_node {
                        AnchorNode::StartOfString => success = split_at == 0,
                        AnchorNode::EndOfString => success = split_at >= string_bytes.len(),
                        AnchorNode::BeginningOfLine => success = split_at == 0 || string_bytes[split_at - 1] == b'\n',
                        AnchorNode::EndOfLine => success = split_at >= string_bytes.len() || string_bytes[split_at] == b'\n',
                        AnchorNode::WordBoundary => {
                            success = (split_at == 0
                                && match string_bytes.get(split_at).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                })
                                || (split_at == string_bytes.len()
                                    && match string_bytes.get(split_at - 1).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                        Some(b) => b,
                                        None => false,
                                    })
                                || (match string_bytes.get(split_at - 1).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                } && !(match string_bytes.get(split_at).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                }))
                                || (match string_bytes.get(split_at).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                } && !(match string_bytes.get(split_at - 1).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                }))
                        }
                        AnchorNode::NotWordBoundary => {
                            success = !((split_at == 0
                                && match string_bytes.get(split_at).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                })
                                || (split_at == string_bytes.len()
                                    && match string_bytes.get(split_at - 1).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                        Some(b) => b,
                                        None => false,
                                    })
                                || (match string_bytes.get(split_at - 1).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                } && !(match string_bytes.get(split_at).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                }))
                                || (match string_bytes.get(split_at).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                } && !(match string_bytes.get(split_at - 1).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                })))
                        }
                    }
                    if success {
                        match &node.children {
                            Children::Multiple(vec) => vec.iter().for_each(|v| {current_stack.insert(*v);}),
                            Children::Single(s) => {current_stack.insert(*s);},
                            Children::None => panic!("Anchor node has no children"),
                        }
                    }
                }
                CNode::Behaviour(_) => {
                    match &node.children {
                        Children::Multiple(vec) => vec.iter().for_each(|v| {current_stack.insert(*v);}),
                        Children::Single(s) => {current_stack.insert(*s);},
                        Children::None => panic!("Behaviour node has no children"),
                    }
                }
                CNode::Special(_) => panic!("Special Nodes not supported on the BFS engine"),
                CNode::End => return Some((string_index, split_at)),
            };
        }
        if to_add_stack.is_empty() {
            if string_index < string_bytes.len() - 1 {
                string_index = next_utf8(string_bytes, string_index);
                to_add_stack.insert(start_node_index);
                split_at = string_index;
            } else {
                return None;
            }
        }
        // println!("{:?}", to_add_stack);
        stack_alt = !stack_alt;

        if let Some((_, len)) = cached {
            split_at += len;
        }
    }
    // None
}

pub(crate) fn indices_match(nodes: &[CompiledNode], string_bytes: &[u8], start_node_index: usize) -> Vec<(usize, usize)> {
    // println!("Breadth First Search engine invoked");
    let mut stack1 = CustomHashSet::default();
    stack1.insert(start_node_index);
    let mut stack2 = CustomHashSet::default();
    let mut stack_alt = true;
    let mut string_index = 0usize;
    let mut split_at = 0usize;
    let mut output = Vec::new();
    '_outer: loop {
        let cached: Option<(char, usize)> = decode_utf8(&string_bytes[split_at..]);
        let (current_stack, to_add_stack) = if stack_alt {
            (&mut stack1, &mut stack2)
        } else {
            (&mut stack2, &mut stack1)
        };
        'inner: while let Some(node_idx) = current_stack.pop() {
            let node = unsafe { nodes.get_unchecked(node_idx) };
            // let node = nodes.get(node_idx).unwrap();
            // println!("{:?}", node);
            let success: bool;
            match &node.node {
                CNode::Match(match_node) => match cached {
                    Some(t) => {
                        let character = &t.0;
                        match match_node {
                            MatchNode::One(match_node) => match match_node {
                                One::MatchOne(c) => success = character == c,
                                One::NotMatchOne(c) => success = character != c,
                                One::MatchAll => success = true,
                            },
                            MatchNode::Range(match_node) => match match_node {
                                Range::Inclusive(chars) => success = chars.contains(character),
                                Range::Exclusive(chars) => success = !chars.contains(character),
                                Range::InclusiveRange(ranges) => {
                                    success = ranges.iter().find(|(start, end)| character >= start && character <= end).is_some()
                                }
                                Range::ExclusiveRange(ranges) => {
                                    success = ranges.iter().find(|(start, end)| character >= start && character <= end).is_none()
                                }
                            },
                        }
                        if success {
                            match &node.children {
                                Children::Multiple(vec) => vec.iter().for_each(|v| {to_add_stack.insert(*v);}),
                                Children::Single(s) => {to_add_stack.insert(*s);},
                                Children::None => panic!("Match node has no children"),
                            }
                        }
                    }
                    None => continue 'inner,
                },
                CNode::Anchor(anchor_node) => {
                    use crate::constants::*;
                    match anchor_node {
                        AnchorNode::StartOfString => success = split_at == 0,
                        AnchorNode::EndOfString => success = split_at >= string_bytes.len(),
                        AnchorNode::BeginningOfLine => success = split_at == 0 || string_bytes[split_at - 1] == b'\n',
                        AnchorNode::EndOfLine => success = split_at >= string_bytes.len() || string_bytes[split_at] == b'\n',
                        AnchorNode::WordBoundary => {
                            success = (split_at == 0
                                && match string_bytes.get(split_at).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                })
                                || (split_at == string_bytes.len()
                                    && match string_bytes.get(split_at - 1).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                        Some(b) => b,
                                        None => false,
                                    })
                                || (match string_bytes.get(split_at - 1).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                } && !(match string_bytes.get(split_at).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                }))
                                || (match string_bytes.get(split_at).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                } && !(match string_bytes.get(split_at - 1).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                }))
                        }
                        AnchorNode::NotWordBoundary => {
                            success = !((split_at == 0
                                && match string_bytes.get(split_at).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                })
                                || (split_at == string_bytes.len()
                                    && match string_bytes.get(split_at - 1).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                        Some(b) => b,
                                        None => false,
                                    })
                                || (match string_bytes.get(split_at - 1).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                } && !(match string_bytes.get(split_at).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                }))
                                || (match string_bytes.get(split_at).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                } && !(match string_bytes.get(split_at - 1).map(|c| W_BYTES.binary_search(c).is_ok()) {
                                    Some(b) => b,
                                    None => false,
                                })))
                        }
                    }
                    if success {
                        match &node.children {
                            Children::Multiple(vec) => vec.iter().for_each(|v| {current_stack.insert(*v);}),
                            Children::Single(s) => {current_stack.insert(*s);},
                            Children::None => panic!("Anchor node has no children"),
                        }
                    }
                }
                CNode::Behaviour(_) => {
                    match &node.children {
                        Children::Multiple(vec) => vec.iter().for_each(|v| {current_stack.insert(*v);}),
                        Children::Single(s) => {current_stack.insert(*s);},
                        Children::None => (),
                    }
                }
                CNode::Special(_) => panic!("Special Nodes not supported on the BFS engine"),
                CNode::End => {
                    output.push((string_index, split_at));
                    string_index = split_at;
                    to_add_stack.clear();
                    current_stack.clear();
                },
            };
        }
        if to_add_stack.is_empty() {
            if string_index < string_bytes.len() - 1 {
                string_index = next_utf8(string_bytes, string_index);
                to_add_stack.insert(start_node_index);
                split_at = string_index;
            } else {
                return output;
            }
        }
        // println!("{:?}", to_add_stack);
        stack_alt = !stack_alt;

        if let Some((_, len)) = cached {
            split_at += len;
        }
    }
    // None
}