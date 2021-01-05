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
    DropStack,
}

#[derive(Clone, Debug)]
pub(crate) enum SpecialNode {
    End,
    Fail,
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
    Range(Range)
}
#[derive(Clone, Debug)]
pub enum One {
    MatchOne(char),
    NotMatchOne(char),
    MatchAll,
}
#[derive(Clone, Debug)]
pub enum Range {
    InclusiveRange(Vec<(char, char)>),
    Inclusive(SortedVec<char>),
    Exclusive(SortedVec<char>),
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

impl CompiledNode {
    pub fn compile(nodes: Vec<super::nfa::Node>) -> (Vec<CompiledNode>, usize) {
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

        for (i, node) in nodes.into_iter().enumerate() {
            if referenced.contains(&i) {
                cnodes.push(node.to_cnode(&old_to_new));
            }
        }

        // println!("{:?}", cnodes);
        (cnodes, start)
    }
}

pub trait Find<T> {
    fn find(&self, other: &T) -> bool;
}

impl Find<char> for Vec<(char, char)> {
    fn find(&self, other: &char) -> bool {
        for (start, end) in self {
            if other >= start && other <= end {
                return true;
            }
        }
        false
    }
}