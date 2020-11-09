#[derive(Clone, Debug)]
pub(crate) enum Node {
    // The children are pointers to indices in the vector of nodes
    Inclusive { children: Vec<usize>, characters: Vec<char> },
    Exclusive { children: Vec<usize>, characters: Vec<char> },
    End,
    MatchAll { children: Vec<usize> },
    Transition { children: Vec<usize>, behaviour: BehaviourNode },
    BeginningOfLine { children: Vec<usize> },
    EndOfLine { children: Vec<usize> },
    MatchOne { children: Vec<usize>, character: char },
    NotMatchOne { children: Vec<usize>, character: char },
}
// Behavioural Nodes, change behaviour of transition nodes if one is attached

#[derive(Clone, Debug)]
pub(crate) enum BehaviourNode {
    None,
    EndOfGroup,
    LookAhead,
    EndLookAhead,
    LookBehind(usize),
    VariableSizeLookBehind(usize, usize),
    CaptureOn,
    CaptureOff,
    DropStack,
}

// Constructors for convenience
impl Node {
    #[inline]
    pub fn new_transition() -> Self {
        return Node::Transition {
            children: vec![],
            behaviour: BehaviourNode::None,
        };
    }

    #[inline]
    pub fn new_behavioural_transition(b: BehaviourNode) -> Self {
        Node::Transition {
            children: vec![],
            behaviour: b,
        }
    }

    #[inline]
    pub fn new_match_all() -> Self {
        return Node::MatchAll { children: vec![] };
    }

    #[inline]
    pub fn new_from_char(c: char, exclude: bool) -> Self {
        if exclude {
            return Node::NotMatchOne {
                children: vec![],
                character: c,
            };
        } else {
            return Node::MatchOne {
                children: vec![],
                character: c,
            };
        }
    }

    #[inline]
    pub fn new_from_chars(chars: Vec<char>, exclude: bool) -> Self {
        if exclude {
            return Node::Exclusive {
                children: vec![],
                characters: chars,
            };
        } else {
            return Node::Inclusive {
                children: vec![],
                characters: chars,
            };
        }
    }

    #[inline]
    pub fn new_end_of_line() -> Self {
        return Node::EndOfLine { children: Vec::new() };
    }

    #[inline]
    pub fn new_start_of_line() -> Self {
        return Node::BeginningOfLine { children: Vec::new() };
    }
}
