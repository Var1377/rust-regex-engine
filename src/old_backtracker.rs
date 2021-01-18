pub(crate) fn c_pure_match(nodes: &[CompiledNode], string: &[char], callstack: &mut Vec<(usize, usize, usize, bool)>, start_node: usize) -> bool {
    let mut recurstion_stack: Vec<usize> = Vec::new();
    let mut completed_recurstion_stack: Vec<usize> = Vec::new();
    let mut lookahead_stack: Vec<usize> = Vec::new();
    let mut completed_lookahead_stack = Vec::<usize>::new();
    callstack.push((start_node, 0usize, 0, true));
    let mut start_index = 0;
    let len = string.len();
    // let mut current_captures = fxhash::FxHashMap::<u32, Vec<usize>>::default();
    // let mut completed_captures = CapturesMap::default();
    loop {
        match callstack.last() {
            None => {
                start_index += 1;
                if start_index < len {
                    callstack.push((start_node, 0usize, start_index, true));
                } else {
                    return false;
                }
            }
            Some(x) => {
                let (node_index, child, string_index, just_inserted) = *x;
                let node = unsafe {nodes.get_unchecked(node_index)};
                match &node.node {
                    Match(match_node) => {
                        if just_inserted {
                            if string.get(string_index).map(|c| match_node.is_match(c)).is_true() {
                                callstack.pop();
                                continue
                            }
                        }
                        match &node.children {
                            Children::Multiple(children) => {
                                let (_,c,_,b) = callstack.last_mut().unwrap();
                                *b = false;
                                match children.get(*c) {
                                    None => {
                                        callstack.pop();
                                    },
                                    Some(n) => {
                                        *c += 1;
                                        callstack.push((*n, 0, string_index + 1, true));
                                    }
                                }
                            }
                            Children::Single(child) => {
                                callstack.pop();
                                callstack.push((*child, 0, string_index + 1, true));
                            }
                            Children::None => panic!("Found no children on match node")
                        }
                    }
                    Anchor(anchor_node) => {
                        if just_inserted && !anchor_node.is_match_chars(string_index, string) {
                            callstack.pop();
                            continue
                        }
                        match &node.children {
                            Children::Multiple(children) => {
                                let (_,c,_,b) = callstack.last_mut().unwrap();
                                *b = false;
                                match children.get(*c) {
                                    None => {
                                        callstack.pop();
                                    },
                                    Some(n) => {
                                        *c += 1;
                                        callstack.push((*n, 0, string_index, true));
                                    }
                                }
                            }
                            Children::Single(child) => {
                                callstack.pop();
                                callstack.push((*child, 0, string_index, true));
                            }
                            Children::None => panic!("Found no children on match node")
                        }
                    }
                    Behaviour(_) => {
                        use BehaviourNode::*;
                        match &node.children {
                            Children::Multiple(children) => {
                                let (_,c,_,b) = callstack.last_mut().unwrap();
                                *b = false;
                                match children.get(*c) {
                                    None => {
                                        callstack.pop();
                                    },
                                    Some(n) => {
                                        *c += 1;
                                        callstack.push((*n, 0, string_index, true));
                                    }
                                }
                            }
                            Children::Single(child) => {
                                callstack.pop();
                                callstack.push((*child, 0, string_index, true));
                            }
                            Children::None => panic!("Found no children on transition node")
                        }
                    }
                    Special(special_node) => {
                        use SpecialNode::*;
                        let consider_children;
                        callstack.last_mut().unwrap().3 = false;
                        match special_node {
                            DropStack => {
                                consider_children = true;
                                callstack.clear();
                                callstack.push((node_index, child, string_index, just_inserted));
                            } 
                            GlobalRecursion => {
                                consider_children = false;
                                callstack.pop();
                            }
                            Subroutine => unimplemented!("Subroutine not implemented"),
                            StartLookAhead => {
                                consider_children = true;
                                if just_inserted {
                                    lookahead_stack.push(string_index);
                                }
                            }
                            EndLookAhead => {
                                consider_children = false;
                                if just_inserted {
                                    completed_lookahead_stack.push(lookahead_stack.pop().unwrap());
                                }
                                match &node.children {
                                    Children::Multiple(children) => {
                                        let (_,c,_,b) = callstack.last_mut().unwrap();
                                        *b = false;
                                        match children.get(*c) {
                                            None => {
                                                lookahead_stack.push(completed_lookahead_stack.pop().unwrap());
                                                callstack.pop();
                                            },
                                            Some(n) => {
                                                *c += 1;
                                                callstack.push((*n, 0, *completed_lookahead_stack.last().unwrap(), true));
                                            }
                                        }
                                    }
                                    Children::Single(child) => {
                                        if just_inserted {
                                            callstack.push((*child, 0,  *completed_lookahead_stack.last().unwrap(), true));
                                        } else {
                                            lookahead_stack.push(completed_lookahead_stack.pop().unwrap());
                                            callstack.pop();
                                        }
                                    }
                                    Children::None => panic!("Found no children on special node")
                                }
                            }
                            StartNegativeLookAhead => {
                                consider_children = false;
                                if !just_inserted {
                                    match &node.children {
                                        Children::Multiple(children) => {
                                            match children.get(child) {
                                                Some(n) => {
                                                    let (_, child, _, just_inserted) = callstack.last_mut().unwrap();
                                                    *child += 1;
                                                    *just_inserted = false;
                                                    callstack.push((*n, 0, string_index, true));
                                                }
                                                None => {
                                                    callstack.pop();
                                                    callstack.push((node_index + 1, 0, string_index, false));
                                                }
                                            }
                                        }
                                        Children::Single(_) => {
                                            callstack.pop();
                                            callstack.push((node_index + 1, 0, string_index, false));
                                        }
                                        Children::None => panic!()
                                    }
                                } else {
                                    match &node.children {
                                        Children::Multiple(children) => {
                                            let (_,c,_,b) = callstack.last_mut().unwrap();
                                            *b = false;
                                            match children.get(*c) {
                                                None => {
                                                    callstack.pop();
                                                },
                                                Some(n) => {
                                                    *c += 1;
                                                    callstack.push((*n, 0, string_index, true));
                                                }
                                            }
                                        }
                                        Children::Single(child) => {
                                            callstack.push((*child, 0, string_index, true));
                                        }
                                        Children::None => panic!("Found no children on special node")
                                    }
                                }
                            }
                            EndNegativeLookAhead => {
                                if just_inserted {
                                    consider_children = false;
                                    loop {
                                        match callstack.pop() {
                                            Some(x) => {
                                                let (before_index, _, _, _) = x;
                                                if before_index == node_index - 1 {
                                                    break;
                                                }
                                            }
                                            None => {
                                                callstack.clear();
                                            }
                                        }
                                    }
                                } else {
                                    consider_children = true;
                                }
                            }
                            _ => unimplemented!(),
                        }
                        if consider_children {
                            match &node.children {
                                Children::Multiple(children) => {
                                    let (_,c,_,b) = callstack.last_mut().unwrap();
                                    *b = false;
                                    match children.get(*c) {
                                        None => {
                                            callstack.pop();
                                        },
                                        Some(n) => {
                                            *c += 1;
                                            callstack.push((*n, 0, string_index, true));
                                        }
                                    }
                                }
                                Children::Single(child) => {
                                    callstack.pop();
                                    callstack.push((*child, 0, string_index, true));
                                }
                                Children::None => panic!("Found no children on special node")
                            }
                        }
                    }
                    End => {
                        if just_inserted {
                            match recurstion_stack.pop() {
                                Some(x) => {
                                    completed_recurstion_stack.push(x);
                                    callstack.push((x, 0, string_index, false));
                                }
                                None => {
                                    callstack.clear();
                                    // completed_captures.insert(0, vec![(start_index, string_index)]);
                                    // return Some(completed_captures);
                                    return true;
                                }
                            }
                        } else {
                            callstack.pop();
                            match completed_recurstion_stack.pop() {
                                Some(x) => {
                                    recurstion_stack.push(x);
                                    callstack.pop();
                                }
                                None => panic!("Not just inserted"),
                            }
                        }
                        continue
                    }
                }
            }
        }
    }
}


pub(crate) fn c_index_match(nodes: &[CompiledNode], string: &[(usize, char)], callstack: &mut Vec<(usize, usize, usize, bool)>, start_node: usize) -> Option<(usize, usize)> {
    let mut recurstion_stack: Vec<usize> = Vec::new();
    let mut completed_recurstion_stack: Vec<usize> = Vec::new();
    let mut lookahead_stack: Vec<usize> = Vec::new();
    let mut completed_lookahead_stack = Vec::<usize>::new();
    let mut start_index = 0;
    let len = string.len();
    callstack.push((start_node, 0usize, start_index, true));
    // let mut current_captures = fxhash::FxHashMap::<u32, Vec<usize>>::default();
    // let mut completed_captures = CapturesMap::default();
    loop {
        match callstack.last() {
            None => {
                start_index += 1;
                if start_index < len {
                    callstack.push((start_node, 0usize, start_index, true));
                } else {
                    return None;
                }
            }
            Some(x) => {
                let (node_index, child, string_index, just_inserted) = *x;
                let node = unsafe {nodes.get_unchecked(node_index)};
                match &node.node {
                    Match(match_node) => {
                        if just_inserted {
                            let c = match string.get(string_index) {
                                Some(c) => c,
                                None => {
                                    callstack.pop();
                                    continue
                                },
                            };
                            let current_char = &c.1;
                            if !match_node.is_match(current_char) {
                                callstack.pop();
                                continue
                            }
                        }
                        match &node.children {
                            Children::Multiple(children) => {
                                let (_,c,_,b) = callstack.last_mut().unwrap();
                                *b = false;
                                match children.get(*c) {
                                    None => {
                                        callstack.pop();
                                    },
                                    Some(n) => {
                                        *c += 1;
                                        callstack.push((*n, 0, string_index + 1, true));
                                    }
                                }
                            }
                            Children::Single(child) => {
                                callstack.pop();
                                callstack.push((*child, 0, string_index + 1, true));
                            }
                            Children::None => panic!("Found no children on match node")
                        }
                    }
                    Anchor(anchor_node) => {
                        if just_inserted {
                            if !anchor_node.is_match_char_indices(string_index, string) {
                                callstack.pop();
                                continue
                            }
                        }
                        match &node.children {
                            Children::Multiple(children) => {
                                let (_,c,_,b) = callstack.last_mut().unwrap();
                                *b = false;
                                match children.get(*c) {
                                    None => {
                                        callstack.pop();
                                    },
                                    Some(n) => {
                                        *c += 1;
                                        callstack.push((*n, 0, string_index, true));
                                    }
                                }
                            }
                            Children::Single(child) => {
                                callstack.pop();
                                callstack.push((*child, 0, string_index, true));
                            }
                            Children::None => panic!("Found no children on match node")
                        }
                    }
                    Behaviour(_) => {
                        use BehaviourNode::*;
                        match &node.children {
                            Children::Multiple(children) => {
                                let (_,c,_,b) = callstack.last_mut().unwrap();
                                *b = false;
                                match children.get(*c) {
                                    None => {
                                        callstack.pop();
                                    },
                                    Some(n) => {
                                        *c += 1;
                                        callstack.push((*n, 0, string_index, true));
                                    }
                                }
                            }
                            Children::Single(child) => {
                                callstack.pop();
                                callstack.push((*child, 0, string_index, true));
                            }
                            Children::None => panic!("Found no children on transition node")
                        }
                    }
                    Special(special_node) => {
                        use SpecialNode::*;
                        let consider_children;
                        callstack.last_mut().unwrap().3 = false;
                        match special_node {
                            DropStack => {
                                consider_children = true;
                                callstack.clear();
                                callstack.push((node_index, child, string_index, just_inserted));
                            } 
                            GlobalRecursion => {
                                consider_children = false;
                                callstack.pop();
                            }
                            Subroutine => unimplemented!("Subroutine not implemented"),
                            StartLookAhead => {
                                consider_children = true;
                                if just_inserted {
                                    lookahead_stack.push(string_index);
                                }
                            }
                            EndLookAhead => {
                                consider_children = false;
                                if just_inserted {
                                    completed_lookahead_stack.push(lookahead_stack.pop().unwrap());
                                }
                                match &node.children {
                                    Children::Multiple(children) => {
                                        let (_,c,_,b) = callstack.last_mut().unwrap();
                                        *b = false;
                                        match children.get(*c) {
                                            None => {
                                                lookahead_stack.push(completed_lookahead_stack.pop().unwrap());
                                                callstack.pop();
                                            },
                                            Some(n) => {
                                                *c += 1;
                                                callstack.push((*n, 0, *completed_lookahead_stack.last().unwrap(), true));
                                            }
                                        }
                                    }
                                    Children::Single(child) => {
                                        if just_inserted {
                                            callstack.push((*child, 0,  *completed_lookahead_stack.last().unwrap(), true));
                                        } else {
                                            lookahead_stack.push(completed_lookahead_stack.pop().unwrap());
                                            callstack.pop();
                                        }
                                    }
                                    Children::None => panic!("Found no children on special node")
                                }
                            }
                            StartNegativeLookAhead => {
                                consider_children = false;
                                if !just_inserted {
                                    match &node.children {
                                        Children::Multiple(children) => {
                                            match children.get(child) {
                                                Some(n) => {
                                                    let (_, child, _, just_inserted) = callstack.last_mut().unwrap();
                                                    *child += 1;
                                                    *just_inserted = false;
                                                    callstack.push((*n, 0, string_index, true));
                                                }
                                                None => {
                                                    let (a,b,c,d) = callstack.last_mut().unwrap();
                                                    *a = node_index + 1;
                                                    *b = 0;
                                                    *c = string_index;
                                                    *d = false;
                                                }
                                            }
                                        }
                                        Children::Single(_) => {
                                            callstack.pop();
                                            callstack.push((node_index + 1, 0, string_index, false));
                                        }
                                        Children::None => panic!()
                                    }
                                } else {
                                    match &node.children {
                                        Children::Multiple(children) => {
                                            let (_,c,_,b) = callstack.last_mut().unwrap();
                                            *b = false;
                                            match children.get(*c) {
                                                None => {
                                                    callstack.pop();
                                                },
                                                Some(n) => {
                                                    *c += 1;
                                                    callstack.push((*n, 0, string_index, true));
                                                }
                                            }
                                        }
                                        Children::Single(child) => {
                                            callstack.push((*child, 0, string_index, true));
                                        }
                                        Children::None => panic!("Found no children on special node")
                                    }
                                }
                            }
                            EndNegativeLookAhead => {
                                if just_inserted {
                                    consider_children = false;
                                    loop {
                                        match callstack.pop() {
                                            Some(x) => {
                                                let (before_index, _, _, _) = x;
                                                if before_index == node_index - 1 {
                                                    break;
                                                }
                                            }
                                            None => {
                                                callstack.clear();
                                            }
                                        }
                                    }
                                } else {
                                    consider_children = true;
                                }
                            }
                            _ => unimplemented!(),
                        }
                        if consider_children {
                            match &node.children {
                                Children::Multiple(children) => {
                                    let (_,c,_,b) = callstack.last_mut().unwrap();
                                    *b = false;
                                    match children.get(*c) {
                                        None => {
                                            callstack.pop();
                                        },
                                        Some(n) => {
                                            *c += 1;
                                            callstack.push((*n, 0, string_index, true));
                                        }
                                    }
                                }
                                Children::Single(child) => {
                                    callstack.pop();
                                            callstack.push((*child, 0, string_index, true));
                                }
                                Children::None => panic!("Found no children on special node")
                            }
                        }
                    }
                    End => {
                        if just_inserted {
                            match recurstion_stack.pop() {
                                Some(x) => {
                                    completed_recurstion_stack.push(x);
                                    callstack.push((x, 0, string_index, false));
                                }
                                None => {
                                    callstack.clear();
                                    // completed_captures.insert(0, vec![(start_index, string_index)]);
                                    // return Some(completed_captures);
                                    return Some((unsafe {string.get_unchecked(start_index).0}, string_index));
                                }
                            }
                        } else {
                            callstack.pop();
                            match completed_recurstion_stack.pop() {
                                Some(x) => {
                                    recurstion_stack.push(x);
                                    callstack.pop();
                                }
                                None => panic!("Not just inserted"),
                            }
                        }
                        continue
                    }
                }
            }
        }
    }
}


pub(crate) fn c_indices_match(nodes: &[CompiledNode], string: &[(usize, char)], callstack: &mut Vec<(usize, usize, usize, bool)>, start_node: usize) -> Vec<(usize, usize)> {
    let mut recurstion_stack: Vec<usize> = Vec::new();
    let mut completed_recurstion_stack: Vec<usize> = Vec::new();
    let mut lookahead_stack: Vec<usize> = Vec::new();
    let mut completed_lookahead_stack = Vec::<usize>::new();
    let mut start_index = 0;
    let len = string.len();
    callstack.push((start_node, 0usize, start_index, true));
    // let mut current_captures = fxhash::FxHashMap::<u32, Vec<usize>>::default();
    // let mut completed_captures = CapturesMap::default();
    let mut matches = Vec::new();
    loop {
        match callstack.last() {
            None => {
                start_index += 1;
                if start_index < len {
                    callstack.push((start_node, 0usize, start_index, true));
                } else {
                    return matches;
                }
            }
            Some(x) => {
                let (node_index, child, string_index, just_inserted) = *x;
                let node = unsafe {nodes.get_unchecked(node_index)};
                match &node.node {
                    Match(match_node) => {
                        if just_inserted {
                            let c = match string.get(string_index) {
                                Some(c) => c,
                                None => {
                                    callstack.pop();
                                    continue
                                },
                            };
                            let current_char = &c.1;
                            if !match_node.is_match(current_char) {
                                callstack.pop();
                                continue
                            }
                        }
                        match &node.children {
                            Children::Multiple(children) => {
                                let (_,c,_,b) = callstack.last_mut().unwrap();
                                *b = false;
                                match children.get(*c) {
                                    None => {
                                        callstack.pop();
                                    },
                                    Some(n) => {
                                        *c += 1;
                                        callstack.push((*n, 0, string_index + 1, true));
                                    }
                                }
                            }
                            Children::Single(child) => {
                                callstack.pop();
                                callstack.push((*child, 0, string_index + 1, true));
                            }
                            Children::None => panic!("Found no children on match node")
                        }
                    }
                    Anchor(anchor_node) => {
                        if just_inserted && !anchor_node.is_match_char_indices(string_index, string) {
                            callstack.pop();
                            continue
                        }
                        match &node.children {
                            Children::Multiple(children) => {
                                let (_,c,_,b) = callstack.last_mut().unwrap();
                                *b = false;
                                match children.get(*c) {
                                    None => {
                                        callstack.pop();
                                    },
                                    Some(n) => {
                                        *c += 1;
                                        callstack.push((*n, 0, string_index, true));
                                    }
                                }
                            }
                            Children::Single(child) => {
                                callstack.pop();
                                callstack.push((*child, 0, string_index, true));
                            }
                            Children::None => panic!("Found no children on match node")
                        }
                    }
                    Behaviour(_) => {
                        match &node.children {
                            Children::Multiple(children) => {
                                let (_,c,_,b) = callstack.last_mut().unwrap();
                                *b = false;
                                match children.get(*c) {
                                    None => {
                                        callstack.pop();
                                    },
                                    Some(n) => {
                                        *c += 1;
                                        callstack.push((*n, 0, string_index, true));
                                    }
                                }
                            }
                            Children::Single(child) => {
                                callstack.pop();
                                callstack.push((*child, 0, string_index, true));
                            }
                            Children::None => panic!("Found no children on transition node")
                        }
                    }
                    Special(special_node) => {
                        use SpecialNode::*;
                        let consider_children;
                        callstack.last_mut().unwrap().3 = false;
                        match special_node {
                            DropStack => {
                                consider_children = true;
                                callstack.clear();
                                callstack.push((node_index, child, string_index, just_inserted));
                            } 
                            GlobalRecursion => {
                                consider_children = false;
                                callstack.pop();
                            }
                            Subroutine => unimplemented!("Subroutine not implemented"),
                            StartLookAhead => {
                                consider_children = true;
                                if just_inserted {
                                    lookahead_stack.push(string_index);
                                }
                            }
                            EndLookAhead => {
                                consider_children = false;
                                if just_inserted {
                                    completed_lookahead_stack.push(lookahead_stack.pop().unwrap());
                                }
                                match &node.children {
                                    Children::Multiple(children) => {
                                        let (_,c,_,b) = callstack.last_mut().unwrap();
                                        *b = false;
                                        match children.get(*c) {
                                            None => {
                                                lookahead_stack.push(completed_lookahead_stack.pop().unwrap());
                                                callstack.pop();
                                            },
                                            Some(n) => {
                                                *c += 1;
                                                callstack.push((*n, 0, *completed_lookahead_stack.last().unwrap(), true));
                                            }
                                        }
                                    }
                                    Children::Single(child) => {
                                        if just_inserted {
                                            callstack.push((*child, 0,  *completed_lookahead_stack.last().unwrap(), true));
                                        } else {
                                            lookahead_stack.push(completed_lookahead_stack.pop().unwrap());
                                            callstack.pop();
                                        }
                                    }
                                    Children::None => panic!("Found no children on special node")
                                }
                            }
                            StartNegativeLookAhead => {
                                consider_children = false;
                                if !just_inserted {
                                    match &node.children {
                                        Children::Multiple(children) => {
                                            match children.get(child) {
                                                Some(n) => {
                                                    let (_, child, _, just_inserted) = callstack.last_mut().unwrap();
                                                    *child += 1;
                                                    *just_inserted = false;
                                                    callstack.push((*n, 0, string_index, true));
                                                }
                                                None => {
                                                    callstack.pop();
                                                    callstack.push((node_index + 1, 0, string_index, false));
                                                }
                                            }
                                        }
                                        Children::Single(_) => {
                                            callstack.pop();
                                            callstack.push((node_index + 1, 0, string_index, false));
                                        }
                                        Children::None => panic!()
                                    }
                                } else {
                                    match &node.children {
                                        Children::Multiple(children) => {
                                            let (_,c,_,b) = callstack.last_mut().unwrap();
                                            *b = false;
                                            match children.get(*c) {
                                                None => {
                                                    callstack.pop();
                                                },
                                                Some(n) => {
                                                    *c += 1;
                                                    callstack.push((*n, 0, string_index, true));
                                                }
                                            }
                                        }
                                        Children::Single(child) => {
                                            callstack.push((*child, 0, string_index, true));
                                        }
                                        Children::None => panic!("Found no children on special node")
                                    }
                                }
                            }
                            EndNegativeLookAhead => {
                                if just_inserted {
                                    consider_children = false;
                                    loop {
                                        match callstack.pop() {
                                            Some(x) => {
                                                let (before_index, _, _, _) = x;
                                                if before_index == node_index - 1 {
                                                    break;
                                                }
                                            }
                                            None => {
                                                callstack.clear();
                                            }
                                        }
                                    }
                                } else {
                                    consider_children = true;
                                }
                            }
                            _ => unimplemented!(),
                        }
                        if consider_children {
                            match &node.children {
                                Children::Multiple(children) => {
                                    let (_,c,_,b) = callstack.last_mut().unwrap();
                                    *b = false;
                                    match children.get(*c) {
                                        None => {
                                            callstack.pop();
                                        },
                                        Some(n) => {
                                            *c += 1;
                                            callstack.push((*n, 0, string_index, true));
                                        }
                                    }
                                }
                                Children::Single(child) => {
                                    callstack.pop();
                                    callstack.push((*child, 0, string_index, true));
                                }
                                Children::None => panic!("Found no children on special node")
                            }
                        }
                    }
                    End => {
                        if just_inserted {
                            match recurstion_stack.pop() {
                                Some(x) => {
                                    completed_recurstion_stack.push(x);
                                    callstack.push((x, 0, string_index, false));
                                }
                                None => {
                                    callstack.clear();
                                    // completed_captures.insert(0, vec![(start_index, string_index)]);
                                    // return Some(completed_captures);
                                    matches.push((unsafe {string.get_unchecked(start_index).0}, string_index));
                                    start_index = string_index;
                                }
                            }
                        } else {
                            callstack.pop();
                            match completed_recurstion_stack.pop() {
                                Some(x) => {
                                    recurstion_stack.push(x);
                                    callstack.pop();
                                }
                                None => panic!("Not just inserted"),
                            }
                        }
                        continue
                    }
                }
            }
        }
    }
}

pub(crate) fn c_captures_match(nodes: &[CompiledNode], string: &[char], callstack: &mut Vec<(usize, usize, usize, bool)>, start_index: usize, start_node: usize) -> Option<CapturesMap> {
    let mut recurstion_stack: Vec<usize> = Vec::new();
    let mut completed_recurstion_stack: Vec<usize> = Vec::new();
    let mut lookahead_stack: Vec<usize> = Vec::new();
    let mut completed_lookahead_stack = Vec::<usize>::new();
    callstack.push((start_node, 0usize, start_index, true)); 
    let mut current_captures = fxhash::FxHashMap::<u32, Vec<usize>>::default();
    let mut completed_captures = CapturesMap::default();
    loop {
        match callstack.last() {
            None => {
                callstack.clear();
                return None;
            }
            Some(x) => {
                let (node_index, child, string_index, just_inserted) = *x;
                let node = unsafe {nodes.get_unchecked(node_index)};
                match &node.node {
                    Match(match_node) => {
                        if just_inserted {
                            let current_char = match string.get(string_index) {
                                Some(c) => c,
                                None => {
                                    callstack.pop();
                                    continue
                                },
                            };
                            if !match_node.is_match(current_char) {
                                callstack.pop();
                                continue
                            }
                        }
                        match &node.children {
                            Children::Multiple(children) => {
                                let (_,c,_,b) = callstack.last_mut().unwrap();
                                *b = false;
                                match children.get(*c) {
                                    None => {
                                        callstack.pop();
                                    },
                                    Some(n) => {
                                        *c += 1;
                                        callstack.push((*n, 0, string_index + 1, true));
                                    }
                                }
                            }
                            Children::Single(child) => {
                                callstack.pop();
                                callstack.push((*child, 0, string_index + 1, true));
                            }
                            Children::None => panic!("Found no children on match node")
                        }
                    }
                    Anchor(anchor_node) => {
                        if just_inserted && !anchor_node.is_match_chars(string_index, string) {
                            callstack.pop();
                            continue
                        }
                        match &node.children {
                            Children::Multiple(children) => {
                                let (_,c,_,b) = callstack.last_mut().unwrap();
                                *b = false;
                                match children.get(*c) {
                                    None => {
                                        callstack.pop();
                                    },
                                    Some(n) => {
                                        *c += 1;
                                        callstack.push((*n, 0, string_index, true));
                                    }
                                }
                            }
                            Children::Single(child) => {
                                callstack.pop();
                                callstack.push((*child, 0, string_index, true));
                            }
                            Children::None => panic!("Found no children on match node")
                        }
                    }
                    Behaviour(behaviour_node) => {
                        use BehaviourNode::*;
                        if just_inserted {
                            match behaviour_node {
                                Transition => (),
                                CapGroup(num) => {
                                    match current_captures.get_mut(num) {
                                        None => {current_captures.insert(*num, vec![string_index]);},
                                        Some(vec) => vec.push(string_index),
                                    };
                                }
                                EndCapGroup(num) => {
                                    let start = current_captures.get_mut(num).unwrap().pop().unwrap();
                                    match completed_captures.get_mut(num) {
                                        None => {completed_captures.insert(*num, vec![(start, string_index)]);},
                                        Some(vec) => vec.push((start, string_index)),
                                    };
                                }
                            }
                        }
                        match &node.children {
                            Children::Multiple(children) => {
                                let (_,c,_,b) = callstack.last_mut().unwrap();
                                *b = false;
                                match children.get(*c) {
                                    None => {
                                        callstack.pop();
                                    },
                                    Some(n) => {
                                        *c += 1;
                                        callstack.push((*n, 0, string_index, true));
                                    }
                                }
                            }
                            Children::Single(child) => {
                                callstack.pop();
                                callstack.push((*child, 0, string_index, true));
                            }
                            Children::None => panic!("Found no children on transition node")
                        }
                    }
                    Special(special_node) => {
                        use SpecialNode::*;
                        let consider_children;
                        callstack.last_mut().unwrap().3 = false;
                        match special_node {
                            DropStack => {
                                consider_children = true;
                                callstack.clear();
                                callstack.push((node_index, child, string_index, just_inserted));
                            }
                            GlobalRecursion => {
                                consider_children = false;
                                callstack.pop();
                            }
                            Subroutine => unimplemented!("Subroutine not implemented"),
                            StartLookAhead => {
                                consider_children = true;
                                if just_inserted {
                                    lookahead_stack.push(string_index);
                                }
                            }
                            EndLookAhead => {
                                consider_children = false;
                                if just_inserted {
                                    completed_lookahead_stack.push(lookahead_stack.pop().unwrap());
                                }
                                match &node.children {
                                    Children::Multiple(children) => {
                                        let (_,c,_,b) = callstack.last_mut().unwrap();
                                        *b = false;
                                        match children.get(*c) {
                                            None => {
                                                lookahead_stack.push(completed_lookahead_stack.pop().unwrap());
                                                callstack.pop();
                                            },
                                            Some(n) => {
                                                *c += 1;
                                                callstack.push((*n, 0, *completed_lookahead_stack.last().unwrap(), true));
                                            }
                                        }
                                    }
                                    Children::Single(child) => {
                                        if just_inserted {
                                            callstack.push((*child, 0,  *completed_lookahead_stack.last().unwrap(), true));
                                        } else {
                                            lookahead_stack.push(completed_lookahead_stack.pop().unwrap());
                                            callstack.pop();
                                        }
                                    }
                                    Children::None => panic!("Found no children on special node")
                                }
                            }
                            StartNegativeLookAhead => {
                                consider_children = false;
                                if !just_inserted {
                                    match &node.children {
                                        Children::Multiple(children) => {
                                            match children.get(child) {
                                                Some(n) => {
                                                    let (_, child, _, just_inserted) = callstack.last_mut().unwrap();
                                                    *child += 1;
                                                    *just_inserted = false;
                                                    callstack.push((*n, 0, string_index, true));
                                                }
                                                None => {
                                                    callstack.pop();
                                                    callstack.push((node_index + 1, 0, string_index, false));
                                                }
                                            }
                                        }
                                        Children::Single(_) => {
                                            callstack.pop();
                                            callstack.push((node_index + 1, 0, string_index, false));
                                        }
                                        Children::None => panic!()
                                    }
                                } else {
                                    match &node.children {
                                        Children::Multiple(children) => {
                                            let (_,c,_,b) = callstack.last_mut().unwrap();
                                            *b = false;
                                            match children.get(*c) {
                                                None => {
                                                    callstack.pop();
                                                },
                                                Some(n) => {
                                                    *c += 1;
                                                    callstack.push((*n, 0, string_index, true));
                                                }
                                            }
                                        }
                                        Children::Single(child) => {
                                            callstack.push((*child, 0, string_index, true));
                                        }
                                        Children::None => panic!("Found no children on special node")
                                    }
                                }
                            }
                            EndNegativeLookAhead => {
                                if just_inserted {
                                    consider_children = false;
                                    loop {
                                        match callstack.pop() {
                                            Some(x) => {
                                                let (before_index, _, _, _) = x;
                                                if before_index == node_index - 1 {
                                                    break;
                                                }
                                            }
                                            None => {
                                                callstack.clear();
                                            }
                                        }
                                    }
                                } else {
                                    consider_children = true;
                                }
                            }
                            _ => unimplemented!(),
                        }
                        if consider_children {
                            match &node.children {
                                Children::Multiple(children) => {
                                    let (_,c,_,b) = callstack.last_mut().unwrap();
                                    *b = false;
                                    match children.get(*c) {
                                        None => {
                                            callstack.pop();
                                        },
                                        Some(n) => {
                                            *c += 1;
                                            callstack.push((*n, 0, string_index, true));
                                        }
                                    }
                                }
                                Children::Single(child) => {
                                    callstack.pop();
                                    callstack.push((*child, 0, string_index, true));
                                }
                                Children::None => panic!("Found no children on special node")
                            }
                        }
                    }
                    End => {
                        if just_inserted {
                            match recurstion_stack.pop() {
                                Some(x) => {
                                    completed_recurstion_stack.push(x);
                                    callstack.push((x, 0, string_index, false));
                                }
                                None => {
                                    callstack.clear();
                                    completed_captures.insert(0, vec![(start_index, string_index)]);
                                    return Some(completed_captures);
                                }
                            }
                        } else {
                            callstack.pop();
                            match completed_recurstion_stack.pop() {
                                Some(x) => {
                                    recurstion_stack.push(x);
                                    callstack.pop();
                                }
                                None => panic!("Not just inserted"),
                            }
                        }
                        continue
                    }
                }
            }
        }
    }
}