use super::fnv::FnvHashMap;
use super::nfa::Node::*;
use super::sorted_vec::SortedVec;

trait Offset {
    fn offset(&mut self, num: usize) -> ();
}

impl Offset for Vec<usize> {
    fn offset(&mut self, offset: usize) {
        for i in self.iter_mut() {
            *i += offset;
        }
    }
}

// Faster and more memory efficient
// to put the enum in a tree like structure to reduce the number of branches in a wworst-case scenario
#[derive(Clone, Debug)]
pub(crate) struct CompiledNode {
    pub node: CNode,
    pub children: Children,
}

#[derive(Clone, Debug)]
pub(crate) enum CNode {
    Match(MatchNode),
    Anchor(AnchorNode),
    Behaviour(BehaviourNode),
    Special(SpecialNode),
    End,
}

#[derive(Clone, Debug)]
pub(crate) enum Children {
    Single(usize),
    Multiple(Vec<usize>),
    None,
}

#[derive(Clone, Debug)]
pub(crate) enum BehaviourNode {
    Transition,
    CapGroup(u32),
    EndCapGroup(u32),
}

#[derive(Clone, Debug)]
pub(crate) enum SpecialNode {
    DropStack,
    GlobalRecursion,
    Subroutine,
    StartLookAhead,
    EndLookAhead,
    StartNegativeLookAhead,
    EndNegativeLookAhead,
    StartLookBack(usize),
    EndLookBack,
    StartVariableLookback(usize, usize),
    BackRef(u32),
}

#[derive(Clone, Debug)]
pub(crate) enum MatchNode {
    One(One),
    Range(Range),
}

#[derive(Clone, Debug)]
pub enum One {
    MatchOne(char),
    NotMatchOne(char),
    MatchAll,
}
#[derive(Clone, Debug)]
pub enum Range {
    Inclusive(SortedVec<char>),
    Exclusive(SortedVec<char>),
    InclusiveRange(Vec<(char, char)>),
    ExclusiveRange(Vec<(char, char)>),
}

#[derive(Clone, Debug)]
pub(crate) enum AnchorNode {
    BeginningOfLine,
    EndOfLine,
    WordBoundary,
    NotWordBoundary,
    StartOfString,
    EndOfString,
}

impl AnchorNode {
    pub fn is_match_chars(&self, index: usize, string: &[char]) -> bool {
        return match self {
            Self::StartOfString => index == 0,
            Self::EndOfString => index == string.len(),
            Self::BeginningOfLine => index == 0 || unsafe { string.get_unchecked(index - 1) } == &'\n',
            Self::EndOfLine => index == string.len() || unsafe { string.get_unchecked(index) } == &'\n',
            Self::WordBoundary => {
                (index == 0 && string.get(index).map(|c| c._is_alphanumeric()).is_true())
                    || (index == string.len() && string.get(index - 1).map(|c| c._is_alphanumeric()).is_true())
                    || (string.get(index - 1).map(|c| c._is_alphanumeric()).is_true() && string.get(index).map(|c| c._is_alphanumeric()).is_false())
                    || (string.get(index).map(|c| c._is_alphanumeric()).is_true() && string.get(index - 1).map(|c| c._is_alphanumeric()).is_false())
            }
            Self::NotWordBoundary => {
                !((index == 0 && string.get(index).map(|c| c._is_alphanumeric()).is_true())
                    || (index == string.len() && string.get(index - 1).map(|c| c._is_alphanumeric()).is_true())
                    || (string.get(index - 1).map(|c| c._is_alphanumeric()).is_true() && string.get(index).map(|c| c._is_alphanumeric()).is_false())
                    || (string.get(index).map(|c| c._is_alphanumeric()).is_true() && string.get(index - 1).map(|c| c._is_alphanumeric()).is_false()))
            }
        };
    }

    pub fn is_match_char_indices(&self, index: usize, string: &[(usize, char)]) -> bool {
        return match self {
            Self::StartOfString => index == 0,
            Self::EndOfString => index == string.len(),
            Self::BeginningOfLine => index == 0 || unsafe { string.get_unchecked(index - 1).1 } == '\n',
            Self::EndOfLine => index == string.len() || unsafe { string.get_unchecked(index).1 } == '\n',
            Self::WordBoundary => {
                (index == 0 && string.get(index).map(|c| c.1._is_alphanumeric()).is_true())
                    || (index == string.len() && string.get(index - 1).map(|c| c.1._is_alphanumeric()).is_true())
                    || (string.get(index - 1).map(|c| c.1._is_alphanumeric()).is_true()
                        && string.get(index).map(|c| c.1._is_alphanumeric()).is_false())
                    || (string.get(index).map(|c| c.1._is_alphanumeric()).is_true()
                        && string.get(index - 1).map(|c| c.1._is_alphanumeric()).is_false())
            }
            Self::NotWordBoundary => {
                !((index == 0 && string.get(index).map(|c| c.1._is_alphanumeric()).is_true())
                    || (index == string.len() && string.get(index - 1).map(|c| c.1._is_alphanumeric()).is_true())
                    || (string.get(index - 1).map(|c| c.1._is_alphanumeric()).is_true()
                        && string.get(index).map(|c| c.1._is_alphanumeric()).is_false())
                    || (string.get(index).map(|c| c.1._is_alphanumeric()).is_true()
                        && string.get(index - 1).map(|c| c.1._is_alphanumeric()).is_false()))
            }
        };
    }

    pub fn is_match_bytes(&self, index: usize, length: usize, current_char: Option<char>, previous_char: Option<char>) -> bool {
        return match self {
            Self::StartOfString => index == 0,
            Self::EndOfString => index == length,
            Self::BeginningOfLine => index == 0 || previous_char.map(|c| c == '\n').is_true(),
            Self::EndOfLine => index == length || current_char.map(|c| c == '\n').is_true(),
            Self::WordBoundary => {
                (index == 0 && current_char.map(|c| c._is_alphanumeric()).is_true())
                    || (index == length && previous_char.map(|c| c._is_alphanumeric()).is_true())
                    || (previous_char.map(|c| c._is_alphanumeric()).is_true() && current_char.map(|c| c._is_alphanumeric()).is_false())
                    || (current_char.map(|c| c._is_alphanumeric()).is_true() && previous_char.map(|c| c._is_alphanumeric()).is_false())
            }
            Self::NotWordBoundary => {
                !((index == 0 && current_char.map(|c| c._is_alphanumeric()).is_true())
                    || (index == length && previous_char.map(|c| c._is_alphanumeric()).is_true())
                    || (previous_char.map(|c| c._is_alphanumeric()).is_true() && current_char.map(|c| c._is_alphanumeric()).is_false())
                    || (current_char.map(|c| c._is_alphanumeric()).is_true() && previous_char.map(|c| c._is_alphanumeric()).is_false()))
            }
        };
    }

    pub fn is_match(&self, index: usize, string: &[u8], current: Option<(char, usize)>) -> bool {
        use crate::utf_8::{decode_last_utf8, decode_utf8};
        return match self {
            Self::StartOfString => index == 0,
            Self::EndOfString => index == string.len(),
            Self::BeginningOfLine => index == 0 || decode_last_utf8(&string[..index]).map(|c| c.0 == '\n').is_true(),
            Self::EndOfLine => index == string.len() || current.map(|c| c.0 == '\n').is_true(),
            Self::WordBoundary => {
                (index == 0 && current.map(|c| c.0._is_alphanumeric()).is_true())
                    || (index == string.len() && decode_last_utf8(&string[..index]).map(|c| c.0._is_alphanumeric()).is_true())
                    || (decode_last_utf8(&string[..index]).map(|c| c.0._is_alphanumeric()).is_true()
                        && current.map(|c| c.0._is_alphanumeric()).is_false())
                    || (current.map(|c| c.0._is_alphanumeric()).is_true()
                        && decode_last_utf8(&string[..index]).map(|c| c.0._is_alphanumeric()).is_false())
            }
            Self::NotWordBoundary => {
                !((index == 0 && current.map(|c| c.0._is_alphanumeric()).is_true())
                    || (index == string.len() && decode_last_utf8(&string[..index]).map(|c| c.0._is_alphanumeric()).is_true())
                    || (decode_last_utf8(&string[..index]).map(|c| c.0._is_alphanumeric()).is_true()
                        && current.map(|c| c.0._is_alphanumeric()).is_false())
                    || (current.map(|c| c.0._is_alphanumeric()).is_true()
                        && decode_last_utf8(&string[..index]).map(|c| c.0._is_alphanumeric()).is_false()))
            }
        };
    }
}

use crate::regex::EngineFlag;

impl CompiledNode {
    pub fn compile(nodes: Vec<super::nfa::Node>) -> (Vec<CompiledNode>, usize, EngineFlag) {
        let mut cnodes = Vec::<CompiledNode>::new();

        // For filtering out redundant nodes and therefore cutting down on memory usage
        let mut referenced = SortedVec::new();

        for node in &nodes {
            match node.get_children() {
                None => {}
                Some(children) => {
                    for child in children {
                        referenced.insert(*child);
                    }
                }
            }
        }

        let start: usize;
        if nodes.get(0).unwrap().get_children().unwrap().len() == 1 {
            start = 1;
        } else {
            start = 0;
            referenced.insert(0);
        }

        let mut offset = start;

        let mut old_to_new = FnvHashMap::<usize, usize>::default();

        for i in start..nodes.len() {
            if referenced.contains(&i) {
                old_to_new.insert(i, i - offset);
            } else {
                offset += 1;
            }
        }

        let mut flag = EngineFlag::Backtrack;

        for (i, node) in nodes.into_iter().enumerate() {
            if referenced.contains(&i) {
                let res = node.to_cnode(&old_to_new);
                cnodes.push(res.0);
                if res.1 {
                    flag = EngineFlag::Backtrack;
                }
            }
        }

        // println!("{:?}", cnodes);
        println!("{:?}", flag);
        (cnodes, start, flag)
    }
}

pub trait OptionBool {
    fn is_true(self) -> bool;
    fn is_false(self) -> bool;
}

impl OptionBool for Option<bool> {
    #[inline]
    fn is_true(self) -> bool {
        self.unwrap_or(false)
    }

    #[inline]
    fn is_false(self) -> bool {
        !self.is_true()
    }
}

pub trait CharAlphaNumeric_ {
    fn _is_alphanumeric(&self) -> bool;
}

impl CharAlphaNumeric_ for char {
    #[inline]
    fn _is_alphanumeric(&self) -> bool {
        return self.is_alphanumeric() || self == &'_';
    }
}

pub trait Find<T> {
    fn find(&self, other: &T) -> bool;
}

impl Find<char> for Vec<(char, char)> {
    fn find(&self, target: &char) -> bool {
        if *target < self.first().unwrap().0 || *target > self.last().unwrap().1 {
            return false;
        }

        return self
            .binary_search_by(|v| {
                use std::cmp::Ordering::*;
                let (start, end) = v;
                if target < start {
                    return Less;
                } else if target > end {
                    return Greater;
                } else {
                    return Equal;
                }
            })
            .is_ok();
    }
}

impl MatchNode {
    #[inline]
    pub fn is_match(&self, character: &char) -> bool {
        match self {
            MatchNode::One(match_node) => {
                use self::One::*;
                match match_node {
                    MatchOne(c) => return c == character,
                    NotMatchOne(c) => return c != character,
                    MatchAll => return true,
                }
            }
            MatchNode::Range(match_node) => {
                use self::Range::*;
                match match_node {
                    Inclusive(chars) => return chars.contains(character),
                    Exclusive(chars) => return !chars.contains(character),
                    InclusiveRange(characters) => return characters.find(character),
                    ExclusiveRange(characters) => return characters.find(character),
                }
            }
        }
    }
}
