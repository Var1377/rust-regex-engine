use crate::compiled_node::*;
use crate::root_node_optimizer::RootNode;
use crate::utf_8::*;

// A custom queue is neccessary to provide predictable matching behaviour eg. not finding the shortest match but the first one that would appear in a bracktracker
// This makes lazy and greedy operators do what they were supposed to do

#[derive(Default, Debug, Clone)]
struct Queue(Vec<StackItem>);

impl Queue {
    fn new() -> Self {
        return Default::default();
    }
}

impl core::ops::Deref for Queue {
    type Target = Vec<StackItem>;
    fn deref(&self) -> &Self::Target {
        return &self.0;
    }
}

impl core::ops::DerefMut for Queue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut self.0;
    }
}

impl Queue {
    fn insert(&mut self, other: StackItem) {
        match self.0.binary_search(&other) {
            Ok(i) => {
                if other < self.0[i] {
                    self.0[i] = other;
                }
            }
            Err(i) => {
                self.0.insert(i, other);
            }
        }
    }
}

#[derive(Default, Debug, Clone, PartialOrd, Ord, Eq)]
struct StackItem {
    node_index: usize,
    stacktrace: StackTrace,
}

impl PartialEq for StackItem {
    fn eq(&self, other: &Self) -> bool {
        self.node_index == other.node_index
    }
}

#[derive(Default, Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
struct StackTrace(
    // Node Index of parent, numbered child (lower has higher priority)
    Vec<usize>,
);

impl core::ops::Deref for StackTrace {
    type Target = Vec<usize>;
    fn deref(&self) -> &Self::Target {
        return &self.0;
    }
}

impl core::ops::DerefMut for StackTrace {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut self.0;
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone)]
struct AcceptorState(StackTrace, usize);

pub(crate) fn pure_match(nodes: &[CompiledNode], string_bytes: &[u8], start_node_index: usize, root_node: &Option<RootNode>) -> bool {
    let mut stack1 = sorted_vec::SortedVec::default();
    let mut stack2 = sorted_vec::SortedVec::default();
    let mut stack_alt = true;
    let mut string_index = 0usize;
    let mut split_at = 0usize;

    if let Some(root_node) = root_node {
        match root_node.run(string_bytes, split_at) {
            Some(idx) => {
                stack1.insert(root_node.child);
                split_at = idx;
                string_index = idx;
            }
            None => return false,
        }
    } else {
        stack1.insert(start_node_index);
    }

    // let mut offset = 0;
    '_outer: loop {
        let cached: Option<(char, usize)> = decode_utf8(&string_bytes[split_at..]);
        let (current_stack, to_add_stack) = if stack_alt {
            (&mut stack1, &mut stack2)
        } else {
            (&mut stack2, &mut stack1)
        };
        'inner: while let Some(node_idx) = current_stack.pop()
        {
            let node = unsafe { nodes.get_unchecked(node_idx) };
            // let node = nodes.get(node_idx).unwrap();
            // println!("{:?}", node);
            let success: bool;
            match &node.node {
                CNode::Match(match_node) => match cached {
                    Some(t) => {
                        let character = &t.0;
                        success = match_node.is_match(character);
                        if success {
                            match &node.children {
                                Children::Multiple(vec) => {
                                    for child in vec.iter().copied() {
                                        to_add_stack.insert(child);
                                    }
                                }
                                Children::Single(s) => {
                                    to_add_stack.insert(*s);
                                }
                                Children::None => panic!("Match node has no children"),
                            }
                        }
                    }
                    None => continue 'inner,
                },
                CNode::Anchor(anchor_node) => {
                    success = anchor_node.is_match(split_at, string_bytes, cached);
                    if success {
                        match &node.children {
                            Children::Multiple(vec) => {
                                for child in vec.iter().copied() {
                                    current_stack.insert(child);
                                }
                            }
                            Children::Single(s) => {
                                current_stack.insert(*s);
                            }
                            Children::None => panic!("Anchor node has no children"),
                        }
                    }
                }
                CNode::Behaviour(_) => match &node.children {
                    Children::Multiple(vec) => {
                        for child in vec.iter().copied() {
                            current_stack.insert(child);
                        }
                    }
                    Children::Single(s) => {
                        current_stack.insert(*s);
                    }
                    Children::None => panic!("Behaviour node has no children"),
                },
                CNode::Special(_) => panic!("Special Nodes not supported on the BFS engine"),
                CNode::End => return true,
            };
        }
        if to_add_stack.is_empty() {
            string_index = next_utf8(string_bytes, string_index);
            if string_index < string_bytes.len() {
                if let Some(root_node) = root_node {
                    match root_node.run(string_bytes, split_at) {
                        Some(idx) => {
                            to_add_stack.insert(root_node.child);
                            split_at = idx;
                            string_index = idx;
                        }
                        None => return false,
                    }
                } else {
                    to_add_stack.insert(start_node_index);
                    split_at = string_index;
                }
            } else {
                return false;
            }
        }  
        stack_alt = !stack_alt;

        if let Some((_, len)) = cached {
            split_at += len;
        }
    }
    // None
}

pub(crate) fn index_match(
    nodes: &[CompiledNode],
    string_bytes: &[u8],
    start_node_index: usize,
    root_node: &Option<RootNode>,
) -> Option<(usize, usize)> {
    let mut stack1 = Queue::default();
    let mut stack2 = Queue::default();
    let mut acceptors: Vec<AcceptorState> = Vec::new();
    let mut stack_alt = true;
    let mut string_index = 0usize;
    let mut split_at = 0usize;

    if let Some(root_node) = root_node {
        match root_node.run(string_bytes, split_at) {
            Some(idx) => {
                stack1.insert(StackItem {
                    node_index: root_node.child,
                    ..Default::default()
                });
                split_at = idx;
                string_index = idx;
            }
            None => return None,
        }
    } else {
        stack1.insert(StackItem {
            node_index: start_node_index,
            ..Default::default()
        });
    }

    // let mut offset = 0;
    '_outer: loop {
        let cached: Option<(char, usize)> = decode_utf8(&string_bytes[split_at..]);
        let (current_stack, to_add_stack) = if stack_alt {
            (&mut stack1, &mut stack2)
        } else {
            (&mut stack2, &mut stack1)
        };
        'inner: while let Some(StackItem {
            node_index: node_idx,
            stacktrace,
        }) = current_stack.pop()
        {
            let node = unsafe { nodes.get_unchecked(node_idx) };
            // let node = nodes.get(node_idx).unwrap();
            // println!("{:?}", node);
            let success: bool;
            match &node.node {
                CNode::Match(match_node) => match cached {
                    Some(t) => {
                        let character = &t.0;
                        success = match_node.is_match(character);
                        if success {
                            match &node.children {
                                Children::Multiple(vec) => {
                                    for (index, child) in vec.iter().enumerate() {
                                        let mut stacktrace = stacktrace.clone();
                                        stacktrace.push(index);
                                        to_add_stack.insert(StackItem {
                                            node_index: *child,
                                            stacktrace,
                                        });
                                    }
                                }
                                Children::Single(s) => {
                                    to_add_stack.insert(StackItem { node_index: *s, stacktrace });
                                }
                                Children::None => panic!("Match node has no children"),
                            }
                        }
                    }
                    None => continue 'inner,
                },
                CNode::Anchor(anchor_node) => {
                    success = anchor_node.is_match(split_at, string_bytes, cached);
                    if success {
                        match &node.children {
                            Children::Multiple(vec) => {
                                for (index, child) in vec.iter().enumerate() {
                                    let mut stacktrace = stacktrace.clone();
                                    stacktrace.push(index);
                                    current_stack.insert(StackItem {
                                        node_index: *child,
                                        stacktrace,
                                    });
                                }
                            }
                            Children::Single(s) => {
                                current_stack.insert(StackItem { node_index: *s, stacktrace });
                            }
                            Children::None => panic!("Anchor node has no children"),
                        }
                    }
                }
                CNode::Behaviour(_) => match &node.children {
                    Children::Multiple(vec) => {
                        for (index, child) in vec.iter().enumerate() {
                            let mut stacktrace = stacktrace.clone();
                            stacktrace.push(index);
                            current_stack.insert(StackItem {
                                node_index: *child,
                                stacktrace,
                            });
                        }
                    }
                    Children::Single(s) => {
                        current_stack.insert(StackItem { node_index: *s, stacktrace });
                    }
                    Children::None => panic!("Behaviour node has no children"),
                },
                CNode::Special(_) => panic!("Special Nodes not supported on the BFS engine"),
                CNode::End => {
                    acceptors.push(AcceptorState(stacktrace, split_at));
                },
            };
        }
        if to_add_stack.is_empty() {
            if !acceptors.is_empty() {
                let accepted = acceptors.iter().min().unwrap();
                return Some((string_index, accepted.1));
            }
            string_index = next_utf8(string_bytes, string_index);
            if string_index < string_bytes.len() {
                if let Some(root_node) = root_node {
                    match root_node.run(string_bytes, split_at) {
                        Some(idx) => {
                            to_add_stack.insert(StackItem{node_index: root_node.child, ..Default::default()});
                            split_at = idx;
                            string_index = idx;
                        }
                        None => return None,
                    }
                } else {
                    to_add_stack.insert(StackItem{node_index: start_node_index, ..Default::default()});
                    split_at = string_index;
                }
            } else {
                return None;
            }
        }  
        stack_alt = !stack_alt;

        if let Some((_, len)) = cached {
            split_at += len;
        }
    }
}

pub(crate) fn indices_match(
    nodes: &[CompiledNode],
    string_bytes: &[u8],
    start_node_index: usize,
    root_node: &Option<RootNode>,
) -> Vec<(usize, usize)> {
    let mut stack1 = Queue::default();
    let mut stack2 = Queue::default();
    let mut acceptors: Vec<AcceptorState> = Vec::new();
    let mut stack_alt = true;
    let mut string_index = 0usize;
    let mut split_at = 0usize;

    let mut out = Vec::new();

    if let Some(root_node) = root_node {
        match root_node.run(string_bytes, split_at) {
            Some(idx) => {
                stack1.insert(StackItem {
                    node_index: root_node.child,
                    ..Default::default()
                });
                split_at = idx;
                string_index = idx;
            }
            None => return out,
        }
    } else {
        stack1.insert(StackItem {
            node_index: start_node_index,
            ..Default::default()
        });
    }

    // let mut offset = 0;
    '_outer: loop {
        let cached: Option<(char, usize)> = decode_utf8(&string_bytes[split_at..]);
        let (current_stack, to_add_stack) = if stack_alt {
            (&mut stack1, &mut stack2)
        } else {
            (&mut stack2, &mut stack1)
        };
        'inner: while let Some(StackItem {
            node_index: node_idx,
            stacktrace,
        }) = current_stack.pop()
        {
            let node = unsafe { nodes.get_unchecked(node_idx) };
            let success: bool;
            match &node.node {
                CNode::Match(match_node) => match cached {
                    Some(t) => {
                        let character = &t.0;
                        success = match_node.is_match(character);
                        if success {
                            match &node.children {
                                Children::Multiple(vec) => {
                                    for (index, child) in vec.iter().enumerate() {
                                        let mut stacktrace = stacktrace.clone();
                                        stacktrace.push(index);
                                        to_add_stack.insert(StackItem {
                                            node_index: *child,
                                            stacktrace,
                                        });
                                    }
                                }
                                Children::Single(s) => {
                                    to_add_stack.insert(StackItem { node_index: *s, stacktrace });
                                }
                                Children::None => panic!("Match node has no children"),
                            }
                        }
                    }
                    None => continue 'inner,
                },
                CNode::Anchor(anchor_node) => {
                    success = anchor_node.is_match(split_at, string_bytes, cached);
                    if success {
                        match &node.children {
                            Children::Multiple(vec) => {
                                for (index, child) in vec.iter().enumerate() {
                                    let mut stacktrace = stacktrace.clone();
                                    stacktrace.push(index);
                                    current_stack.insert(StackItem {
                                        node_index: *child,
                                        stacktrace,
                                    });
                                }
                            }
                            Children::Single(s) => {
                                current_stack.insert(StackItem { node_index: *s, stacktrace });
                            }
                            Children::None => panic!("Anchor node has no children"),
                        }
                    }
                }
                CNode::Behaviour(_) => match &node.children {
                    Children::Multiple(vec) => {
                        for (index, child) in vec.iter().enumerate() {
                            let mut stacktrace = stacktrace.clone();
                            stacktrace.push(index);
                            current_stack.insert(StackItem {
                                node_index: *child,
                                stacktrace,
                            });
                        }
                    }
                    Children::Single(s) => {
                        current_stack.insert(StackItem { node_index: *s, stacktrace });
                    }
                    Children::None => panic!("Behaviour node has no children"),
                },
                CNode::Special(_) => panic!("Special Nodes not supported on the BFS engine"),
                CNode::End => {
                    acceptors.push(AcceptorState(stacktrace, split_at));
                },
            };
        }
        if to_add_stack.is_empty() {
            if !acceptors.is_empty() {
                let accepted = acceptors.iter().min().unwrap();
                out.push((string_index, accepted.1));
            }
            string_index = next_utf8(string_bytes, string_index);
            if string_index < string_bytes.len() {
                if let Some(root_node) = root_node {
                    match root_node.run(string_bytes, split_at) {
                        Some(idx) => {
                            to_add_stack.insert(StackItem{node_index: root_node.child, ..Default::default()});
                            split_at = idx;
                            string_index = idx;
                        }
                        None => return out,
                    }
                } else {
                    to_add_stack.insert(StackItem{node_index: start_node_index, ..Default::default()});
                    split_at = string_index;
                }
            } else {
                return out;
            }
        }  
        stack_alt = !stack_alt;

        if let Some((_, len)) = cached {
            split_at += len;
        }
    }
}
