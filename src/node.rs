use std::collections::*;

pub type NodeMap = BTreeMap<usize, Node>;

#[derive(Clone, Debug)]
pub enum Node {
    Inclusive {
        children: Vec<usize>,
        characters: Vec<char>,
        index: usize,
    },
    Exclusive {
        children: Vec<usize>,
        characters: Vec<char>,
        index: usize,
    },
    End,
    MatchAll {
        children: Vec<usize>,
        index: usize,
    },
    Transition {
        children: Vec<usize>,
        index: usize,
    },
}

impl Node {
    pub fn new_transition(index: usize) -> Self {
        return Node::Transition {
            children: vec![],
            index: index,
        };
    }

    pub fn new(index: usize, exclude: bool) -> Self {
        if exclude {
            Node::Exclusive {
                children: vec![],
                characters: vec![],
                index: index,
            }
        } else {
            Node::Inclusive {
                children: vec![],
                characters: vec![],
                index: index,
            }
        }
    }

    pub fn new_match_all(index: usize) -> Self {
        return Node::MatchAll {
            children: vec![],
            index: index,
        };
    }

    pub fn new_from_char(c: char, exclude: bool, index: usize) -> Self {
        if exclude {
            return Node::Exclusive {
                children: vec![],
                characters: vec![c],
                index: index,
            };
        } else {
            return Node::Inclusive {
                children: vec![],
                characters: vec![c],
                index: index,
            };
        }
    }

    pub fn new_from_chars(chars: Vec<char>, exclude: bool, index: usize) -> Self {
        if exclude {
            return Node::Exclusive {
                children: vec![],
                characters: chars,
                index: index,
            };
        } else {
            return Node::Inclusive {
                children: vec![],
                characters: chars,
                index: index,
            };
        }
    }
}
