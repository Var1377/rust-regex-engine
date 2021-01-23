// Enum matching is a constant time operation so I'm taking as much advantage of it as possible by integrating the usual branches in the matching sequence to just the enum match by having a huge variety of nodes.
// Code is not as ergonomic but it is fast.

use super::compiled_node::*;
use super::sorted_vec::*;
use fnv::FnvHashSet;
use Node::*;

#[derive(Clone, Debug)]
pub(crate) enum Node {
    // Main node for matching specific characters
    MatchOne {
        children: Vec<usize>,
        character: char,
    },
    // The children are pointers to indices in the vector of nodes
    Inclusive {
        children: Vec<usize>,
        characters: Vec<char>,
    },
    Exclusive {
        children: Vec<usize>,
        characters: Vec<char>,
    },
    // Unicode Range. The ranges are inclusive
    InclusiveRange {
        characters: Vec<(char, char)>,

        children: Vec<usize>,
    },
    ExclusiveRange {
        characters: Vec<(char, char)>,

        children: Vec<usize>,
    },
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
    // Ending node
    End,
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
        children: Vec<usize>,
    },
    StartLookBack {
        children: Vec<usize>,
        length: usize,
    },
    // No idea how to implement this efficiently :/
    StartVariableLookBack {
        children: Vec<usize>,
        start: usize,
        end: usize,
    },
    // Backreferences, likely to be lazily evaluated because most if a pattern uses it they're not likely to be too concerned about performance. Also probably exclusive to the backtracking engine.
    BackRef {
        children: Vec<usize>,
        number: u32,
    },
    // For possessive quantifiers and atomic groups
    DropStack {
        children: Vec<usize>,
    },
    StartAtomic {
        children: Vec<usize>,
    },
    EndAtomic {
        children: Vec<usize>
    },
    // Recursion
    GlobalRecursion,
}

impl Node {
    #[inline]
    pub const fn new_transition() -> Self {
        return Transition { children: Vec::new() };
    }

    #[inline]
    pub const fn new_match_all() -> Self {
        return MatchAll { children: Vec::new() };
    }

    #[inline]
    pub fn new_from_char(c: char) -> Self {
        return MatchOne {
            children: vec![],
            character: c,
        };
    }

    #[inline]
    pub fn new_from_chars(chars: Vec<char>, exclude: bool) -> Self {
        if exclude {
            return Exclusive {
                children:Vec::new(),
                characters: chars,
            };
        } else {
            return Inclusive {
                children: Vec::new(),
                characters: chars,
            };
        }
    }

    #[inline]
    pub const fn new_end_of_line() -> Self {
        return EndOfLine { children: Vec::new() };
    }

    #[inline]
    pub const fn new_start_of_line() -> Self {
        return BeginningOfLine { children: Vec::new() };
    }

    #[inline]
    pub fn get_children_mut(&mut self) -> Option<&mut Vec<usize>> {
        match self {
            Inclusive { ref mut children, .. }
            | Exclusive { ref mut children, .. }
            | Transition { ref mut children, .. }
            | BeginningOfLine { ref mut children }
            | EndOfLine { ref mut children }
            | MatchOne { ref mut children, .. }
            | MatchAll { ref mut children }
            | BackRef { ref mut children, .. }
            | CapGroup { ref mut children, .. }
            | DropStack { ref mut children }
            | EndCapGroup { ref mut children, .. }
            | EndLookAhead { ref mut children, .. }
            | StartLookAhead { ref mut children, .. }
            | StartLookBack { ref mut children, .. }
            | StartVariableLookBack { ref mut children, .. }
            | WordBoundary { ref mut children }
            | BackRef { ref mut children, .. }
            | BeginningOfString { ref mut children }
            | EndOfString { ref mut children }
            | InclusiveRange { ref mut children, .. }
            | ExclusiveRange { ref mut children, .. }
            | MatchAllandNL { ref mut children }
            | ExclusiveNodes { ref mut children, .. }
            | NotWordBoundary { ref mut children, .. }
            | StartNegativeLookAhead { ref mut children, .. }
            | EndNegativeLookAhead { ref mut children, .. }
            | StartAtomic {ref mut children}
            | EndAtomic {ref mut children} => {
                return Some(children);
            }
            _ => return None,
        };
    }

    #[inline]
    pub fn get_children(&self) -> Option<&Vec<usize>> {
        match self {
            Inclusive { children, .. }
            | Exclusive { children, .. }
            | Transition { children, .. }
            | BeginningOfLine { children }
            | EndOfLine { children }
            | MatchOne { children, .. }
            | MatchAll { children }
            | BackRef { children, .. }
            | CapGroup { children, .. }
            | DropStack { children }
            | EndCapGroup { children, .. }
            | EndLookAhead { children, .. }
            | StartLookAhead { children, .. }
            | StartLookBack { children, .. }
            | StartVariableLookBack { children, .. }
            | WordBoundary { children }
            | BackRef { children, .. }
            | BeginningOfString { children }
            | EndOfString { children }
            | InclusiveRange { children, .. }
            | ExclusiveRange { children, .. }
            | MatchAllandNL { children }
            | ExclusiveNodes { children, .. }
            | NotWordBoundary { children, .. }
            | StartNegativeLookAhead { children, .. }
            | EndNegativeLookAhead { children, .. } 
            | StartAtomic {children}
            | EndAtomic {children} => {
                return Some(children);
            }
            _ => return None,
        }
    }

    #[inline]
    pub fn push_child(&mut self, to_add: usize) {
        let children = self.get_children_mut().unwrap();
        children.push(to_add);
    }

    #[inline]
    pub fn insert_child(&mut self, to_add: usize) {
        let children = self.get_children_mut().unwrap();
        children.insert(0, to_add);
    }

    pub fn lazy_dependent_insert(&mut self, to_add: usize, lazy: bool) {
        match self.get_children_mut() {
            Some(x) => {
                if lazy {
                    x.insert(0, to_add);
                } else {
                    x.push(to_add);
                }
            }
            None => (),
        }
    }

    #[inline]
    pub fn get_transition_children_mut(&mut self) -> &mut Vec<usize> {
        match self {
            Transition { ref mut children, .. }
            | CapGroup { ref mut children, .. }
            | EndCapGroup { ref mut children, .. }
            | StartLookAhead { ref mut children, .. }
            | EndLookAhead { ref mut children, .. }
            | StartLookBack { ref mut children, .. }
            | StartVariableLookBack { ref mut children, .. }
            | BackRef { ref mut children, .. }
            | DropStack { ref mut children, .. }
            | EndNegativeLookAhead { ref mut children, .. }
            | StartAtomic {ref mut children}
            | EndAtomic {ref mut children} => {
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

    pub fn to_cnode(self, old_to_new: &fnv::FnvHashMap<usize, usize>) -> (CompiledNode, bool) {
        let children: Children = match self.get_children() {
            Some(c) => {
                if c.len() == 1 {
                    Children::Single(
                        *old_to_new
                            .get(c.get(0).expect("No items in children"))
                            .expect(&format!("{} not in old_to_new", c.get(0).unwrap())),
                    )
                } else {
                    let vec = c
                        .iter()
                        .map(|v| {
                            return *old_to_new.get(v).unwrap();
                        })
                        .collect();
                    // super::utils::remove_duplicates_without_sort(&mut vec);
                    Children::Multiple(vec)
                }
            }
            None => Children::None,
        };

        use crate::utf_8::CharLen;
        use crate::utils::RangeUtils;

        let node: CNode = match self {
            MatchOne { character, .. } => CNode::Match(MatchNode::One(One::MatchOne(character))),
            Inclusive { characters, .. } => {
                if characters.len() == 1 {
                    CNode::Match(MatchNode::One(One::MatchOne(characters[0])))
                } else {
                    CNode::Match(MatchNode::Range(Range::Inclusive(SortedVec::from(characters))))
                }
            }
            Exclusive { characters, .. } => {
                if characters.len() == 1 {
                    CNode::Match(MatchNode::One(One::NotMatchOne(characters[0])))
                } else {
                    CNode::Match(MatchNode::Range(Range::Exclusive(SortedVec::from(characters))))
                }
            }
            InclusiveRange { mut characters, .. } => {
                characters.minimize();
                CNode::Match(MatchNode::Range(Range::InclusiveRange(characters)))
            }
            ExclusiveRange { mut characters, .. } => {
                characters.minimize();
                CNode::Match(MatchNode::Range(Range::ExclusiveRange(characters)))
            }
            MatchAll { .. } => CNode::Match(MatchNode::One(One::NotMatchOne('\n'))),
            MatchAllandNL { .. } => CNode::Match(MatchNode::One(One::MatchAll)),
            BeginningOfLine { .. } => CNode::Anchor(AnchorNode::BeginningOfLine),
            EndOfLine { .. } => CNode::Anchor(AnchorNode::EndOfLine),
            BeginningOfString { .. } => CNode::Anchor(AnchorNode::StartOfString),
            EndOfString { .. } => CNode::Anchor(AnchorNode::EndOfString),
            WordBoundary { .. } => CNode::Anchor(AnchorNode::WordBoundary),
            NotWordBoundary { .. } => CNode::Anchor(AnchorNode::NotWordBoundary),
            End => CNode::End,
            Transition { .. } => CNode::Behaviour(BehaviourNode::Transition),
            ExclusiveNodes { .. } => unimplemented!(),
            CapGroup { number, .. } => CNode::Behaviour(BehaviourNode::CapGroup(number)),
            StartLookAhead { .. } => CNode::Special(SpecialNode::StartLookAhead),
            EndLookAhead { .. } => CNode::Special(SpecialNode::EndLookAhead),
            StartNegativeLookAhead { .. } => CNode::Special(SpecialNode::StartNegativeLookAhead),
            EndNegativeLookAhead { .. } => CNode::Special(SpecialNode::EndNegativeLookAhead),
            StartLookBack { length, .. } => CNode::Special(SpecialNode::StartLookBack(length)),
            StartVariableLookBack { start, end, .. } => CNode::Special(SpecialNode::StartVariableLookback(start, end)),
            BackRef { number, .. } => CNode::Special(SpecialNode::BackRef(number)),
            DropStack { .. } => CNode::Special(SpecialNode::DropStack),
            EndCapGroup { number, .. } => CNode::Behaviour(BehaviourNode::EndCapGroup(number)),
            GlobalRecursion { .. } => CNode::Special(SpecialNode::GlobalRecursion),
            StartAtomic {..} => CNode::Special(SpecialNode::StartAtomic),
            EndAtomic {..}=> CNode::Special(SpecialNode::EndAtomic),
        };

        let special = match &node {
            CNode::Special(_) => true,
            _ => false,
        };

        return (CompiledNode { children, node }, special);
    }

    pub fn to_start_atomic(&mut self) {
        match self {
            Self::Transition{children} => {
                *self = Node::StartAtomic {children: children.clone()};
            }
            _ => panic!()
        }
    }

    pub fn to_end_atomic(&mut self) {
        match self {
            Self::Transition{children} => {
                *self = Node::EndAtomic {children: children.clone()};
            }
            _ => panic!()
        }
    }

    // pub fn optimize(&mut self) {
    //     if let Some(c) = self.get_children_mut() {
    //         if c.len() == 1 {
    //             match self {
    //                 MatchOne { children, character } => {
    //                     *self = MatchOneX {
    //                         character: *character,
    //                         child: children[0],
    //                     }
    //                 }
    //                 NotMatchOne { children, character } => {
    //                     *self = NotMatchOneX {
    //                         character: *character,
    //                         child: children[0],
    //                     }
    //                 }
    //                 Inclusive { children, characters } => {
    //                     *self = InclusiveX {
    //                         child: children[0],
    //                         characters: characters.clone(),
    //                     }
    //                 }
    //                 Exclusive { children, characters } => {
    //                     *self = ExclusiveX {
    //                         child: children[0],
    //                         characters: characters.clone(),
    //                     }
    //                 }
    //                 MatchTwo {
    //                     children,
    //                     character,
    //                     character2,
    //                 } => {
    //                     *self = MatchTwoX {
    //                         child: children[0],
    //                         character: *character,
    //                         character2: *character2,
    //                     }
    //                 }
    //                 NotMatchTwo {
    //                     children,
    //                     character,
    //                     character2,
    //                 } => {
    //                     *self = NotMatchTwoX {
    //                         child: children[0],
    //                         character: *character,
    //                         character2: *character2,
    //                     }
    //                 }
    //                 InclusiveRange { children, start, end } => {
    //                     *self = InclusiveRangeX {
    //                         child: children[0],
    //                         start: *start,
    //                         end: *end,
    //                     }
    //                 }
    //                 ExclusiveRange { children, start, end } => {
    //                     *self = ExclusiveRangeX {
    //                         child: children[0],
    //                         start: *start,
    //                         end: *end,
    //                     }
    //                 }
    //                 MatchAll { children } => *self = MatchAllX { child: children[0] },
    //                 MatchAllandNL { children } => *self = MatchAllandNLX { child: children[0] },
    //                 BeginningOfLine { children } => *self = BeginningOfLineX { child: children[0] },
    //                 EndOfLine { children } => *self = EndOfLineX { child: children[0] },
    //                 BeginningOfString { children } => *self = BeginningOfStringX { child: children[0] },
    //                 WordBoundary { children } => *self = WordBoundaryX { child: children[0] },
    //                 NotWordBoundary { children } => *self = NotWordBoundaryX { child: children[0] },
    //                 _ => (),
    //             }
    //         }
    //     }
    // }
}

pub trait LazyDependentInsert {
    fn lazy_dependent_insert(&mut self, to_add: usize, lazy: bool);
}

impl LazyDependentInsert for Vec<usize> {
    fn lazy_dependent_insert(&mut self, to_add: usize, lazy: bool) {
        if lazy {
            self.insert(0, to_add);
        } else {
            self.push(to_add);
        }
    }
}