use crate::compiled_node::{One, *};
use crate::utils::RangeUtils;

// Brings the core loop spent over most of the characters as low as possible with as few branches as possible
// Skips traversing the enum tree and checking if the callstack is empty
// Slightly reduces performance on regexes with high success rates
// But greatly increases it on regexes with low success rates, making it worth it

#[derive(Debug, Clone)]
pub(crate) struct RootNode {
    pub node: CNode,
    pub child: usize,
    advance_on_match: bool,
}

impl RootNode {
    pub fn generate(nodes: &[CompiledNode], start: usize, children: Option<usize>) -> Option<Self> {
        let start_node = nodes.get(start)?;

        use CNode::*;
        match &start_node.node {
            Match(_) => match &start_node.children {
                Children::Single(child) => {
                    return Some(Self {
                        node: start_node.node.clone(),
                        advance_on_match: children.as_ref().map(|_| false).unwrap_or(true),
                        child: children.unwrap_or(*child),
                    })
                }
                _ => {
                    return Some(Self {
                        node: start_node.node.clone(),
                        advance_on_match: false,
                        child: children.unwrap_or(start),
                    })
                }
            },
            Behaviour(_) => {
                match &start_node.children {
                    Children::Multiple(child) => {
                        let nodes = child.iter().map(|c| nodes.get(*c).unwrap()).collect::<Vec<_>>();
                        let mut match_nodes = vec![];
                        for node in nodes {
                            match &node.node {
                                CNode::Match(m) => (match_nodes.push(m)),
                                _ => return None,
                            }
                        }

                        let mut match_characters = vec![];
                        let mut match_ranges: Vec<(char, char)> = vec![];
                        let mut no_match_characters = vec![];
                        let mut no_match_ranges: Vec<(char, char)> = vec![];

                        for node in match_nodes {
                            match node {
                                MatchNode::One(node) => match node {
                                    One::MatchOne(c) => match_characters.push(*c),
                                    One::NotMatchOne(c) => no_match_characters.push(*c),
                                    One::MatchAll => return None,
                                },
                                MatchNode::Range(node) => match node {
                                    Range::Inclusive(chars) => match_characters.extend(chars.iter().copied()),
                                    Range::Exclusive(chars) => no_match_characters.extend(chars.iter().copied()),
                                    Range::InclusiveRange(range) => match_ranges.extend(range),
                                    Range::ExclusiveRange(range) => no_match_ranges.extend(range),
                                },
                            }
                        }

                        if !no_match_characters.is_empty() && match_characters.is_empty() && no_match_ranges.is_empty() && match_ranges.is_empty() {
                            return Some(Self {
                                advance_on_match: false,
                                child: children.unwrap_or(start),
                                node: CNode::Match(MatchNode::Range(Range::Exclusive(crate::sorted_vec::SortedVec::from(
                                    no_match_characters,
                                )))),
                            });
                        }

                        for c in no_match_characters {
                            no_match_ranges.push((c, c));
                        }

                        no_match_ranges.invert();
                        match_ranges.extend(no_match_ranges);
                        match_ranges.minimize();

                        // Character only cost
                        use std::iter::Sum;
                        let ranges_sum = u32::sum(match_ranges.iter().map(|v| v.1 as u32 - v.0 as u32));
                        let c_cost = (ranges_sum as f32 + match_characters.len() as f32).log2().ceil() as usize;
                        // Ranges only cost
                        let r_cost = ((match_ranges.len() + match_characters.len()) as f32 * 1.5f32).log2().ceil() as usize;

                        // Approximate cost to leave out the root node
                        let d_cost = 16
                            + (((match_ranges.len() * 2) as f32).log2().ceil() as usize)
                            + (((match_characters.len()) as f32).log2().ceil() as usize);

                        // println!("Range cost: {}, charset cost: {}, none cost: {}",r_cost, c_cost, d_cost);

                        if d_cost < c_cost && d_cost < r_cost {
                            return None;
                        } else if r_cost < c_cost {
                            for character in match_characters {
                                match_ranges.push((character, character));
                            }
                            match_ranges.minimize();

                            return Some(Self {
                                node: CNode::Match(MatchNode::Range(Range::InclusiveRange(match_ranges))),
                                advance_on_match: false,
                                child: children.unwrap_or(start),
                            });
                        } else {
                            for (start, end) in match_ranges {
                                (start..=end).for_each(|v| match_characters.push(v));
                            }
                            return Some(Self {
                                node: CNode::Match(MatchNode::Range(Range::Inclusive(crate::sorted_vec::SortedVec::from(match_characters)))),
                                advance_on_match: false,
                                child: children.unwrap_or(start),
                            });
                        }
                    }
                    Children::Single(child) => {
                        return Self::generate(nodes, *child, Some(children.unwrap_or(start)));
                    }
                    _ => return None,
                }
            }
            Anchor(_) => match &start_node.children {
                Children::Single(child) => {
                    return Some(Self {
                        node: start_node.node.clone(),
                        advance_on_match: false,
                        child: children.unwrap_or(*child),
                    })
                }
                _ => {
                    return Some(Self {
                        node: start_node.node.clone(),
                        advance_on_match: false,
                        child: children.unwrap_or(start),
                    })
                }
            },
            _ => return None,
        }
    }

    #[inline(always)]
    pub fn run(&self, string: &[u8], mut index: usize) -> Option<usize> /* String index */ {
        use crate::utf_8::*;
        use CNode::*;
        match &self.node {
            Match(match_node) => {
                match match_node {
                    MatchNode::One(match_node) => match match_node {
                        One::MatchOne(c) => {
                            let c = *c;
                            while index < string.len() {
                                if let Some((character, len)) = decode_utf8(&string[index..]) {
                                    if character == c {
                                        if self.advance_on_match {
                                            index += len;
                                        }
                                        return Some(index);
                                    } else {
                                        index += len;
                                    }
                                } else {
                                    return None;
                                }
                            }
                        }
                        One::NotMatchOne(c) => {
                            let c = *c;
                            while index < string.len() {
                                if let Some((character, len)) = decode_utf8(&string[index..]) {
                                    if character != c {
                                        if self.advance_on_match {
                                            index += len;
                                        }
                                        return Some(index);
                                    } else {
                                        index += len;
                                    }
                                } else {
                                    return None;
                                }
                            }
                        }
                        _ => unreachable!(),
                    },
                    MatchNode::Range(match_node) => match match_node {
                        Range::Inclusive(chars) => {
                            while index < string.len() {
                                if let Some((character, len)) = decode_utf8(&string[index..]) {
                                    if chars.contains(&character) {
                                        if self.advance_on_match {
                                            index += len;
                                        }
                                        return Some(index);
                                    } else {
                                        index += len;
                                    }
                                } else {
                                    return None;
                                }
                            }
                        }
                        Range::Exclusive(chars) => {
                            while index < string.len() {
                                if let Some((character, len)) = decode_utf8(&string[index..]) {
                                    if !chars.contains(&character) {
                                        if self.advance_on_match {
                                            index += len;
                                        }
                                        return Some(index);
                                    } else {
                                        index += len;
                                    }
                                } else {
                                    return None;
                                }
                            }
                        }
                        Range::InclusiveRange(ranges) => {
                            while index < string.len() {
                                if let Some((character, len)) = decode_utf8(&string[index..]) {
                                    if ranges.find(&character) {
                                        if self.advance_on_match {
                                            index += len;
                                        }
                                        return Some(index);
                                    }
                                    index += len;
                                } else {
                                    return None;
                                }
                            }
                        }
                        Range::ExclusiveRange(ranges) => {
                            while index < string.len() {
                                if let Some((character, len)) = decode_utf8(&string[index..]) {
                                    if !ranges.find(&character) {
                                        if self.advance_on_match {
                                            index += len;
                                        }
                                        return Some(index);
                                    }
                                    index += len;
                                } else {
                                    return None;
                                }
                            }
                        }
                    },
                }
                return None;
            }
            Anchor(anchor_node) => match anchor_node {
                AnchorNode::WordBoundary => {
                    if index > string.len() {
                        return None;
                    }
                    let character = decode_utf8(&string[index..]);
                    if index == 0 {
                        if let Some((character, len)) = character {
                            if character._is_alphanumeric() {
                                return Some(0);
                            } else {
                                index += len;
                            }
                        } else {
                            return None;
                        }
                    }

                    let mut last_character = decode_last_utf8(&string[..index]);
                    while index < string.len() {
                        if let Some((new_character, len)) = decode_utf8(&string[index..]) {
                            if let Some((last_character, _)) = last_character {
                                if (new_character._is_alphanumeric() && !last_character._is_alphanumeric())
                                    || (!new_character._is_alphanumeric() && last_character._is_alphanumeric())
                                {
                                    return Some(index);
                                }
                            } else if let Some((last_character, _)) = decode_last_utf8(&string[..index]) {
                                if (new_character._is_alphanumeric() && !last_character._is_alphanumeric())
                                    || (!new_character._is_alphanumeric() && last_character._is_alphanumeric())
                                {
                                    return Some(index);
                                }
                            } else {
                                return None;
                            }
                            index += len;
                            last_character = Some((new_character, len));
                        } else {
                            return None;
                        }
                    }
                    if index == string.len() {
                        if let Some((c, _)) = last_character {
                            if c._is_alphanumeric() {
                                return Some(index);
                            }
                        }
                        return None;
                    }
                }
                AnchorNode::NotWordBoundary => {
                    if index > string.len() {
                        return None;
                    }
                    let character = decode_utf8(&string[index..]);
                    if index == 0 {
                        if let Some((character, len)) = character {
                            if !character._is_alphanumeric() {
                                return Some(0);
                            } else {
                                index += len;
                            }
                        } else {
                            return None;
                        }
                    }

                    let mut last_character = decode_last_utf8(&string[..index]);
                    while index < string.len() {
                        if let Some((new_character, len)) = decode_utf8(&string[index..]) {
                            if let Some((last_character, _)) = last_character {
                                if !((new_character._is_alphanumeric() && !last_character._is_alphanumeric())
                                    || (!new_character._is_alphanumeric() && last_character._is_alphanumeric()))
                                {
                                    return Some(index);
                                }
                            } else if let Some((last_character, _)) = decode_last_utf8(&string[..index]) {
                                if !((new_character._is_alphanumeric() && !last_character._is_alphanumeric())
                                    || (!new_character._is_alphanumeric() && last_character._is_alphanumeric()))
                                {
                                    return Some(index);
                                }
                            } else {
                                return None;
                            }
                            index += len;
                            last_character = Some((new_character, len));
                        } else {
                            return None;
                        }
                    }
                    if index == string.len() {
                        if let Some((c, _)) = last_character {
                            if !c._is_alphanumeric() {
                                return Some(index);
                            }
                        }
                        return None;
                    }
                }
                AnchorNode::StartOfString => {
                    if index == 0 {
                        return Some(0);
                    } else {
                        return None;
                    }
                }
                AnchorNode::EndOfString => {
                    if index <= string.len() {
                        return Some(string.len());
                    } else {
                        return None;
                    }
                }
                AnchorNode::BeginningOfLine => {
                    if index == 0 {
                        return Some(0);
                    }
                    let mut last_character: Option<(char, usize)> = None;
                    while index < string.len() {
                        if let Some((character, _)) = last_character {
                            if character == '\n' {
                                return Some(index);
                            }
                        } else if let Some((character, _)) = decode_last_utf8(&string[..index]) {
                            if character == '\n' {
                                return Some(index);
                            }
                        } else {
                            return None;
                        }

                        if let Some(t) = decode_utf8(&string[index..]) {
                            index += t.1;
                            last_character = Some(t);
                        } else {
                            return None;
                        }
                    }
                }
                AnchorNode::EndOfLine => {
                    if index == string.len() {
                        return Some(string.len());
                    }
                    while index < string.len() {
                        if let Some((character, len)) = decode_utf8(&string[..index]) {
                            if character == '\n' {
                                return Some(index);
                            } else {
                                index += len;
                            }
                        } else {
                            return None;
                        }
                    }
                }
            },
            _ => unreachable!("Only match and anchor nodes supported in the root node"),
        }
        None
    }
}
