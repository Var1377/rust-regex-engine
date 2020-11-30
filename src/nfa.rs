// Enum matching is a constant time operation so I'm taking as much advantage of it as possible by integrating the usual branches in the matching sequence to just the enum match by having a huge variety of nodes.
// Code is not as ergonomic but it is fast. 

use fxhash::FxHashSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Node {
    // Main node for matching specific characters
    MatchOne {
        children: Vec<usize>,
        character: char,
    },
    // When an exclusive character class has only one member eg. [^a]
    NotMatchOne {
        children: Vec<usize>,
        character: char,
    },
    // The children are pointers to indices in the vector of nodes
    Inclusive {
        children: Vec<usize>,
        characters: FxHashSet<char>,
    },
    Exclusive {
        children: Vec<usize>,
        characters: FxHashSet<char>,
    },
    // Usually used for case insensitivity. eg. character: 'a', character2: 'A'. Quicker than using ^
    MatchTwo {
        children: Vec<usize>,
        character: char,
        character2: char,
    },
    NotMatchTwo {
        children: Vec<usize>,
        character: char,
        character2: char,
    },
    // Unicode Range. The ranges are inclusive
    InclusiveUnicodeRange {
        start: u32,
        end: u32,
        children: Vec<usize>,
    },
    ExclusiveUnicodeRange {
        start: u32,
        end: u32,
        children: Vec<usize>,
    },
    // Ending node
    End,
    // Used on negative lookarounds
    Fail,
    // . character - if it matches a newline or not based on the config object
    MatchAll {
        children: Vec<usize>,
    },
    MatchAllandNL {
        children: Vec<usize>,
    },
    // Anchors
    BeginningOfLine {
        children: Vec<usize>,
    },
    EndOfLine {
        children: Vec<usize>,
    },
    BeginningOfString {
        children: Vec<usize>,
    },
    EndOfString {
        children: Vec<usize>,
    },
    // \b and \B
    WordBoundary {
        children: Vec<usize>,
    },
    NotWordBoundary {
        children: Vec<usize>,
    },
    // Epsilon Transition State, Ideally removed by the time it reaches the matching engine.
    Transition {
        children: Vec<usize>,
    },
    // For when a exclusive character class has internal character classes as well as other classes/nodes. It has to fail every match to continue.
    ExclusiveNodes {
        children: Vec<usize>,
        nodes: Vec<usize>,
    },
    // Capturing Groups
    CapGroup {
        children: Vec<usize>,
        number: u32,
    },
    EndCapGroup {
        children: Vec<usize>,
        number: u32,
    },
    // For lookarounds
    StartLookAhead {
        children: Vec<usize>,
    },
    EndLookAhead {
        children: Vec<usize>,
    },
    StartNegativeLookAhead {
        // What to lookahead to
        children: Vec<usize>,
    },
    EndNegativeLookAhead {
        // What comes after the lookahead
        children: Vec<usize>
    },
    StartLookBack {
        children: Vec<usize>,
        length: usize,
    },
    // No idea how to implement this efficiently :/
    StartVariableLookBack {
        children: Vec<usize>,
    },
    // Backreferences, likely to be lazily evaluated as I've never seen anyone actually use these. Also probably exclusive to the backtracking engine.
    BackRef {
        children: Vec<usize>,
        number: u32,
    },
    // For possessive quantifiers and atomic groups
    DropStack {
        children: Vec<usize>,
    },
    // Recursion
    GlobalRecursion,
    CapGroupRecursion {
        children: Vec<usize>,
        num: u32,
    },
    // ((if this matches)Do this|Else Do this)
    // Path two is a fail. If it gets to the end conditional it succeeds
    StartConditional {
        path2: Vec<usize>,
        children: Vec<usize>,
    },
    EndConditional {
        children: Vec<usize>,
    },
}

impl Node {
    #[inline]
    pub fn new_transition() -> Self {
        return Node::Transition { children: vec![] };
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
    pub fn new_from_chars(chars: &[char], exclude: bool) -> Self {
        if exclude {
            return Node::Exclusive {
                children: vec![],
                characters: chars.iter().cloned().collect::<FxHashSet<char>>(),
            };
        } else {
            return Node::Inclusive {
                children: vec![],
                characters: chars.iter().cloned().collect::<FxHashSet<char>>(),
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

    #[inline]
    pub fn get_children_mut(&mut self) -> Option<&mut Vec<usize>> {
        match self {
            Node::End | Node::Fail | Node::GlobalRecursion => return None,
            Node::Inclusive { ref mut children, .. }
            | Node::Exclusive { ref mut children, .. }
            | Node::Transition { ref mut children, .. }
            | Node::BeginningOfLine { ref mut children }
            | Node::EndOfLine { ref mut children }
            | Node::MatchOne { ref mut children, .. }
            | Node::MatchAll { ref mut children }
            | Node::NotMatchOne { ref mut children, .. }
            | Node::BackRef { ref mut children, .. }
            | Node::CapGroup { ref mut children, .. }
            | Node::DropStack { ref mut children }
            | Node::EndCapGroup { ref mut children, .. }
            | Node::EndLookAhead { ref mut children, .. }
            | Node::StartLookAhead { ref mut children, .. }
            | Node::StartLookBack { ref mut children, .. }
            | Node::StartVariableLookBack { ref mut children, .. }
            | Node::WordBoundary { ref mut children }
            | Node::BackRef { ref mut children, .. }
            | Node::BeginningOfString { ref mut children }
            | Node::EndOfString { ref mut children }
            | Node::InclusiveUnicodeRange { ref mut children, .. }
            | Node::ExclusiveUnicodeRange { ref mut children, .. }
            | Node::MatchAllandNL { ref mut children }
            | Node::ExclusiveNodes { ref mut children, .. }
            | Node::CapGroupRecursion { ref mut children, .. }
            | Node::NotWordBoundary { ref mut children, .. }
            | Node::StartConditional { ref mut children, .. }
            | Node::EndConditional { ref mut children }
            | Node::MatchTwo { ref mut children, .. }
            | Node::NotMatchTwo { ref mut children, .. }
            | Node::StartNegativeLookAhead {ref mut children, ..}
            | Node::EndNegativeLookAhead {ref mut children , ..} => {
                return Some(children);
            }
        };
    }

    #[inline]
    pub fn push_child(&mut self, to_add: usize) {
        let mut children = self.get_children_mut().unwrap();
        children.push(to_add);
    }

    #[inline]
    pub fn insert_child(&mut self, to_add: usize) {
        let mut children = self.get_children_mut().unwrap();
        children.insert(0, to_add);
    }

    #[inline]
    pub fn lazy_dependent_insert(&mut self, to_add: usize, lazy: bool) {
        let children = self.get_children_mut().unwrap();
        if lazy {
            children.push(to_add);
        } else {
            children.insert(0, to_add);
        }
    }

    #[inline]
    pub fn get_transition_children_mut(&mut self) -> &mut Vec<usize> {
        match self {
            Node::Transition { ref mut children, .. }
            | Node::CapGroup { ref mut children, .. }
            | Node::EndCapGroup { ref mut children, .. }
            | Node::StartLookAhead { ref mut children, .. }
            | Node::EndLookAhead { ref mut children, .. }
            | Node::StartLookBack { ref mut children, .. }
            | Node::StartVariableLookBack { ref mut children, .. }
            | Node::BackRef { ref mut children, .. }
            | Node::DropStack { ref mut children, .. }
            | Node::EndNegativeLookAhead {ref mut children, ..} => {
                return children;
            }
            _ => panic!(format!("{:?}", self)),
        }
    }

    #[inline]
    pub fn transition_push_child(&mut self, to_add: usize) {
        let children = self.get_transition_children_mut();
        children.push(to_add);
    }

    #[inline]
    pub fn transition_insert_child(&mut self, to_add: usize) {
        let children = self.get_transition_children_mut();
        children.insert(0, to_add);
    }

    #[inline]
    pub fn transition_lazy_dependent(&mut self, to_add: usize, lazy: bool) {
        let children = self.get_transition_children_mut();
        if lazy {
            children.push(to_add);
        } else {
            children.insert(0, to_add);
        }
    }
}
