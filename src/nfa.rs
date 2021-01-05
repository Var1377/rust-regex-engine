// Enum matching is a constant time operation so I'm taking as much advantage of it as possible by integrating the usual branches in the matching sequence to just the enum match by having a huge variety of nodes.
// Code is not as ergonomic but it is fast.

use fnv::FnvHashSet;
use Node::*;
use super::compiled_node::*;
use super::sorted_vec::*;
use derivative::*;

#[derive(Clone, Debug, Eq, Derivative)]
#[derivative(PartialEq, Hash)]
pub(crate) enum Node {
    // Main node for matching specific characters
    MatchOne {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        children: Vec<usize>,
        character: char,
    },
    // The children are pointers to indices in the vector of nodes
    Inclusive {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        children: Vec<usize>,
        characters: Vec<char>,
    },
    Exclusive {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        children: Vec<usize>,
        characters: Vec<char>,
    },
    // Unicode Range. The ranges are inclusive
    InclusiveRange {
        characters: Vec<(char, char)>,
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        children: Vec<usize>,
    },
    ExclusiveRange {
        characters: Vec<(char, char)>,
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        children: Vec<usize>,
    },
    // . character - if it matches a newline or not based on the config object
    MatchAll {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        children: Vec<usize>,
    },
    MatchAllandNL {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        children: Vec<usize>,
    },
    // Anchors
    BeginningOfLine {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        children: Vec<usize>,
    },
    EndOfLine {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        children: Vec<usize>,
    },
    BeginningOfString {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        children: Vec<usize>,
    },
    EndOfString {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        children: Vec<usize>,
    },
    // \b and \B
    WordBoundary {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        children: Vec<usize>,
    },
    NotWordBoundary {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        children: Vec<usize>,
    },
    // Ending node
    End,
    // Used on negative lookarounds
    Fail,
    // Epsilon Transition State, Ideally removed by the time it reaches the matching engine.
    Transition {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        children: Vec<usize>,
    },
    // For when a exclusive character class has internal character classes as well as other classes/nodes. It has to fail every match to continue.
    ExclusiveNodes {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        children: Vec<usize>,
        nodes: Vec<usize>,
    },
    // Capturing Groups
    CapGroup {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        children: Vec<usize>,
        number: u32,
    },
    EndCapGroup {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        children: Vec<usize>,
        number: u32,
    },
    // For lookarounds
    StartLookAhead {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        children: Vec<usize>,
    },
    EndLookAhead {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        children: Vec<usize>,
    },
    StartNegativeLookAhead {
        // What to lookahead to
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        children: Vec<usize>,
    },
    EndNegativeLookAhead {
        // What comes after the lookahead
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        children: Vec<usize>,
    },
    StartLookBack {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        children: Vec<usize>,
        length: usize,
    },
    // No idea how to implement this efficiently :/
    StartVariableLookBack {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        children: Vec<usize>,
        start: usize,
        end: usize
    },
    // Backreferences, likely to be lazily evaluated because most if a pattern uses it they're not likely to be too concerned about performance. Also probably exclusive to the backtracking engine.
    BackRef {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        children: Vec<usize>,
        number: u32,
    },
    // For possessive quantifiers and atomic groups
    DropStack {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        children: Vec<usize>,
    },
    // Recursion
    GlobalRecursion,
}

impl Node {
    #[inline]
    pub fn new_transition() -> Self {
        return Transition { children: vec![] };
    }

    #[inline]
    pub fn new_match_all() -> Self {
        return MatchAll { children: vec![] };
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
                children: vec![],
                characters: chars,
            };
        } else {
            return Inclusive {
                children: vec![],
                characters: chars,
            };
        }
    }

    #[inline]
    pub fn new_end_of_line() -> Self {
        return EndOfLine { children: Vec::new() };
    }

    #[inline]
    pub fn new_start_of_line() -> Self {
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
            | EndNegativeLookAhead { ref mut children, .. } => {
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
            | EndNegativeLookAhead { children, .. } => {
                return Some(children);
            }
            _ => return None,
        }
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
            | EndNegativeLookAhead { ref mut children, .. } => {
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

    pub fn to_cnode(self, old_to_new: &fnv::FnvHashMap<usize,usize>) -> CompiledNode {
        let children: Children = match self.get_children() {
            Some(c) => {
                if c.len() == 1 {
                    Children::Single(*old_to_new.get(c.get(0).expect("No items in children")).expect(&format!("{} not in old_to_new", c.get(0).unwrap())))
                } else {
                    let mut vec = c.iter()
                    .map(|v| {
                        return *old_to_new.get(v).unwrap();
                    })
                    .collect();
                    // super::utils::remove_duplicates_without_sort(&mut vec);
                    Children::Multiple(
                        vec
                    )
                }
            }
            None => Children::None,
        };

        use crate::utf_8::CharLen;

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
            InclusiveRange { characters, .. } => CNode::Match(MatchNode::Range(Range::InclusiveRange(characters))),
            ExclusiveRange { characters, .. } => CNode::Match(MatchNode::Range(Range::ExclusiveRange(characters))),
            MatchAll { .. } => CNode::Match(MatchNode::One(One::NotMatchOne('\n'))),
            MatchAllandNL { .. } => CNode::Match(MatchNode::One(One::MatchAll)),
            BeginningOfLine { .. } => CNode::Anchor(AnchorNode::BeginningOfLine),
            EndOfLine { .. } => CNode::Anchor(AnchorNode::EndOfLine),
            BeginningOfString { .. } => CNode::Anchor(AnchorNode::StartOfString),
            EndOfString { .. } => CNode::Anchor(AnchorNode::EndOfString),
            WordBoundary {..} => CNode::Anchor(AnchorNode::WordBoundary),
            NotWordBoundary {..} => CNode::Anchor(AnchorNode::NotWordBoundary),
            End => CNode::Special(SpecialNode::End),
            Fail => CNode::Special(SpecialNode::Fail),
            Transition {..} => CNode::Behaviour(BehaviourNode::Transition),
            ExclusiveNodes {..} => unimplemented!(),
            CapGroup {number, ..} => CNode::Behaviour(BehaviourNode::CapGroup(number)),
            StartLookAhead {..} => CNode::Special(SpecialNode::StartLookAhead),
            EndLookAhead {..} => CNode::Special(SpecialNode::EndLookAhead),
            StartNegativeLookAhead {..} => CNode::Special(SpecialNode::StartNegativeLookAhead),
            EndNegativeLookAhead {..} => CNode::Special(SpecialNode::EndNegativeLookAhead),
            StartLookBack {length, ..} => CNode::Special(SpecialNode::StartLookBack(length)),
            StartVariableLookBack {start, end, ..} => CNode::Special(SpecialNode::StartVariableLookback(start, end)),
            BackRef {number, ..} => CNode::Special(SpecialNode::BackRef(number)),
            DropStack {..} => CNode::Behaviour(BehaviourNode::DropStack),
            EndCapGroup {number, ..} => CNode::Behaviour(BehaviourNode::EndCapGroup(number)),
            GlobalRecursion {..} => CNode::Special(SpecialNode::GlobalRecursion),
        };

        return CompiledNode {
            children, node
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
