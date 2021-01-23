use super::{compiled_node::OptionBool, constants::*, nfa::*, parse::ParseToken};

pub fn remove_duplicates_without_sort<T: PartialEq + Eq + std::hash::Hash + Copy>(vec: &mut Vec<T>, set: &mut fxhash::FxHashSet<T>) {
    // Linear time complexity and reuses allocations in the set
    let mut i = 0usize;
    while i < vec.len() {
        let num = vec.get(i).unwrap();
        if set.contains(&num) {
            vec.remove(i);
        } else {
            set.insert(*num);
            i += 1;
        }
    }
    set.clear();
    vec.shrink_to_fit();
}

pub trait RangeUtils {
    fn invert(&mut self);
    fn minimize(&mut self);
}

impl RangeUtils for Vec<(char, char)> {
    fn invert(&mut self) {
        let mut new = vec![];
        for (start, end) in self.iter() {
            new.push((0 as char, unsafe { char::from_u32_unchecked((*start as u32).saturating_sub(1)) }));
            new.push((unsafe { char::from_u32_unchecked((*end as u32).saturating_add(1)) }, std::char::MAX));
        }
        // new.minimize();
        *self = new;
    }

    fn minimize(&mut self) {
        if self.is_empty() {
            return;
        }
        self.sort_unstable();
        self.dedup();
        let mut new_ranges = vec![];
        let (mut left, mut right) = self[0];
        for (new_left, new_right) in self.iter().skip(1) {
            if (right as u32) + 1 < *new_left as u32 {
                new_ranges.push((left, right));
                left = *new_left;
                right = *new_right;
            } else {
                right = std::cmp::max(right, *new_right);
            }
        }
        new_ranges.push((left, right));
        *self = new_ranges;
    }
}

pub(crate) fn str_to_char_vec(string: &str) -> Vec<char> {
    let mut vec = Vec::with_capacity(string.len());
    string.chars().for_each(|v| {
        vec.push(v);
    });
    return vec;
}

pub(crate) fn char_vec_to_string(chars: &[char]) -> String {
    chars.iter().collect::<String>()
}

pub fn previous_char_is_closing_bracket(index: &usize, chars: &[char]) -> bool {
    if *index == 0 {
        return false;
    } else {
        let lookback = chars[index - 1];
        if lookback == ')' || lookback == ']' {
            if check_if_escaped(chars, *index - 1) {
                return false;
            }
            return true;
        } else {
            return false;
        }
    }
}

pub(crate) fn parse_range_character(c: char) -> Node {
    match c {
        'd' => {
            return Node::InclusiveRange {
                children: vec![],
                characters: d(),
            }
        }
        'D' => {
            return Node::ExclusiveRange {
                children: vec![],
                characters: d(),
            }
        }
        'w' => {
            return Node::InclusiveRange {
                children: vec![],
                characters: w(),
            };
        }
        'W' => {
            return Node::ExclusiveRange {
                children: vec![],
                characters: w(),
            };
        }
        's' => return Node::new_from_chars(WHITESPACE.to_vec(), false),
        'S' => return Node::new_from_chars(WHITESPACE.to_vec(), true),
        'b' => return Node::WordBoundary { children: vec![] },
        'B' => return Node::NotWordBoundary { children: vec![] },
        _ => panic!("Range character not supported"),
    };
}

pub(crate) fn parse_square_brackets(chars: &mut Vec<char>, node_vec: &mut Vec<Node>, callstack: &mut Vec<ParseToken>) {
    // println!("Square Expression: {:?}", chars);
    if chars.len() == 0 {
        return;
    }
    // let before = Node::new_transition();
    // let before_index = node_vec.len();
    // node_vec.push(before);
    // let after = Node::new_transition();
    // let after_index = node_vec.len();
    // node_vec.push(after);
    let exclusive;
    // let mut nodes = Vec::<Node>::new();
    if chars[0] == '^' {
        exclusive = true;
        chars.remove(0);
    } else {
        exclusive = false;
    }

    let mut ranges = Vec::<(char, char)>::new();
    let mut no_match_ranges = Vec::<(char, char)>::new();
    let mut match_characters = Vec::<char>::new();
    let mut no_match_characters = Vec::<char>::new();

    let mut looking_back = false;

    let mut tokens = vec![];

    for character in chars {
        if looking_back {
            match character {
                '\\' => tokens.push(('\\', false)),
                _ => tokens.push((*character, true)),
            }
            looking_back = false;
        } else {
            match character {
                '\\' => looking_back = true,
                _ => tokens.push((*character, false)),
            }
        }
    }

    let mut i = 0;

    while i < tokens.len() {
        let (character, escaped) = tokens[i];
        if escaped {
            match character {
                'w' => {
                    ranges.push(('a', 'z'));
                    ranges.push(('A', 'Z'));
                    match_characters.push('_');
                }
                'W' => {
                    no_match_ranges.push(('a', 'z'));
                    no_match_ranges.push(('A', 'Z'));
                    no_match_characters.push('_');
                }
                's' => {
                    match_characters.extend(WHITESPACE);
                }
                'S' => {
                    no_match_characters.extend(WHITESPACE);
                }
                'd' => {
                    ranges.push(('0', '9'));
                }
                'D' => {
                    no_match_ranges.push(('0', '9'));
                }
                _ => match_characters.push(character),
            }
        } else {
            if tokens.get(i + 1) == Some(&('-', false)) && tokens.get(i + 2).map(|v| v.1 == false).is_true() {
                // bounds checking already done in the if statement
                let end = tokens[i + 2].0;
                ranges.push((character, end));
                i += 2;
            } else {
                match_characters.push(character);
            }
        }
        i += 1;
    }

    // Deal with no match characters

    for character in no_match_characters {
        no_match_ranges.push((character, character));
    }

    // Deal with no match ranges

    no_match_ranges.invert();
    ranges.append(&mut no_match_ranges);
    std::mem::drop(no_match_ranges);

    if exclusive {
        if ranges.is_empty() && match_characters.is_empty() {
            add_node(
                Node::Exclusive {
                    children: vec![],
                    characters: vec![],
                },
                node_vec,
                callstack,
            );
            return;
        }

        if ranges.is_empty() {
            add_node(
                Node::Exclusive {
                    children: vec![],
                    characters: match_characters,
                },
                node_vec,
                callstack,
            );
            return;
        }

        ranges.invert();
        ranges.minimize();

        if match_characters.is_empty() {
            add_node(
                Node::ExclusiveRange {
                    characters: ranges,
                    children: vec![],
                },
                node_vec,
                callstack,
            );
            return;
        }

        // Determine the number of branches for a miss in exclusive node
        use std::iter::Sum;
        let ranges_sum = u32::sum(ranges.iter().map(|v| v.1 as u32 - v.0 as u32));
        let c_cost = (ranges_sum as f32 + match_characters.len() as f32).log2().ceil() as usize;
        // Determine the cost for putting it all into a range
        let r_cost = ((ranges.len() + match_characters.len()) as f32 * 2f32).log2().ceil() as usize;

        if r_cost < c_cost {
            for character in match_characters {
                ranges.push((character, character));
            }
            // It will get minimized again when it reaches the compilation stage
            add_node(
                Node::ExclusiveRange {
                    children: vec![],
                    characters: ranges,
                },
                node_vec,
                callstack,
            );
        } else {
            for (start, end) in ranges {
                (start..=end).for_each(|v| match_characters.push(v));
            }
            add_node(
                Node::Exclusive {
                    children: vec![],
                    characters: match_characters,
                },
                node_vec,
                callstack,
            );
        }

        return;
    } else {
        ranges.minimize();

        if ranges.is_empty() && match_characters.is_empty() {
            add_node(
                Node::Inclusive {
                    children: vec![],
                    characters: vec![],
                },
                node_vec,
                callstack,
            );
            return;
        }

        // 3 approaches: Only ranges, only characters or two nodes with a branch.
        // This tries to predict the numver of branches each implementation
        //  takes on it's worst case and determines the most efficient route

        // Character only cost
        use std::iter::Sum;
        let ranges_sum = u32::sum(ranges.iter().map(|v| v.1 as u32 - v.0 as u32));
        let c_cost = (ranges_sum as f32 + match_characters.len() as f32).log2().ceil() as usize;
        // Ranges only cost
        let r_cost = ((ranges.len() + match_characters.len()) as f32 * 1.5f32).log2().ceil() as usize;

        // Putting it in two nodes, undesired because it's very inefficient with a large amount of non-matches
        // Therefore we add a constant to prevent all but the most needed cases to use more than one node
        let d_cost = 16 + (((ranges.len() * 2) as f32).log2().ceil() as usize) + (((match_characters.len()) as f32).log2().ceil() as usize);

        // println!("C: {}, R: {}, D: {}")

        if r_cost < c_cost && r_cost < d_cost {
            for character in match_characters {
                ranges.push((character, character));
            }
            // It will get minimized again when it reaches the compilation stage
            add_node(
                Node::InclusiveRange {
                    children: vec![],
                    characters: ranges,
                },
                node_vec,
                callstack,
            );
            return;
        } else if c_cost < d_cost {
            for (start, end) in ranges {
                (start..=end).for_each(|v| match_characters.push(v));
            }
            add_node(
                Node::Inclusive {
                    children: vec![],
                    characters: match_characters,
                },
                node_vec,
                callstack,
            );
            return;
        } else {
            let len = node_vec.len();
            node_vec.get_mut(callstack.pop().unwrap().idx()).unwrap().push_child(len);
            let v = vec![
                Node::Transition { children: vec![len + 1] },
                Node::Transition {
                    children: vec![len + 4, len + 5],
                },
                Node::Transition { children: vec![len + 3] },
                Node::Transition { children: vec![] },
                Node::InclusiveRange {
                    characters: ranges,
                    children: vec![len + 2],
                },
                Node::Inclusive {
                    characters: match_characters,
                    children: vec![len + 2],
                },
            ];
            node_vec.extend(v);

            callstack.push(ParseToken::M {
                first: len,
                before: len + 1,
                after: len + 2,
                last: len + 3,
                to_link: len + 3,
            });
        }
    }
}

pub(crate) fn parse_curly_brackets(contents: &Vec<char>, node_vec: &mut Vec<Node>, callstack: &mut Vec<ParseToken>, lazy: bool, possessive: bool) {
    use ParseToken::*;
    let pos_of_comma = contents.iter().position(|c| *c == ',');
    if let Some(p) = pos_of_comma {
        if p == contents.len() - 1 {
            let mut s1 = String::new();
            for i in 0..p {
                s1.push(contents[i]);
            }
            let to_repeat = s1.parse::<usize>().unwrap() - 1;
            match callstack.pop().unwrap() {
                S(last_index) => {
                    let len = node_vec.len();
                    node_vec.get_mut(last_index).unwrap().get_children_mut().unwrap().push(len);
                    for i in 0..to_repeat {
                        let mut clone = node_vec.get(last_index).unwrap().clone();
                        let children = clone.get_children_mut().unwrap();
                        children.clear();
                        if i != to_repeat - 1 {
                            children.push(node_vec.len() + 1);
                        } else {
                            children.lazy_dependent_insert(node_vec.len(), lazy);
                        }
                        node_vec.push(clone);
                    }
                    callstack.push(S(node_vec.len() - 1));

                    if possessive {
                        let mut last = node_vec.pop().unwrap();
                        let children = last.get_children_mut().unwrap();
                        children.clear();
                        children.push(node_vec.len() + 1);
                        children.push(node_vec.len() + 2);
                        node_vec.push(Node::StartAtomic {children: vec![node_vec.len() + 1]});
                        node_vec.push(last);
                        node_vec.push(Node::EndAtomic {children: vec![]});
                        callstack.pop();
                        callstack.push(S(node_vec.len() - 1));
                    }
                }
                M {
                    mut before,
                    mut after,
                    first,
                    mut last,
                    ..
                } => {
                    for _ in 0..to_repeat {
                        let mut new_nodes = vec![];
                        let new_before_index = node_vec.len();
                        for i in first..node_vec.len() {
                            new_nodes.push(node_vec.get(i).unwrap().clone());
                        }
                        node_vec.get_mut(last).unwrap().get_children_mut().unwrap().push(new_before_index);
                        for i in 0..new_nodes.len() {
                            let len = new_nodes.len();
                            let node = new_nodes.get_mut(i).unwrap();
                            match node.get_children_mut() {
                                Some(c) => {
                                    for child in c {
                                        *child += len;
                                    }
                                }
                                None => continue,
                            }
                        }
                        last += new_nodes.len();
                        after += new_nodes.len();
                        before += new_nodes.len();
                        node_vec.extend(new_nodes);
                    }
                    node_vec.get_mut(after).unwrap().lazy_dependent_insert(before, lazy);
                    callstack.push(M {
                        first,
                        before,
                        after,
                        last,
                        to_link: last,
                    });
                }
            }
        } else {
            let mut s1 = String::new();
            let mut s2 = String::new();
            for i in 0..p {
                s1.push(contents[i])
            }
            for i in p + 1..contents.len() {
                s2.push(contents[i]);
            }
            let int1 = s1.parse::<usize>().unwrap();
            let int2 = s2.parse::<usize>().unwrap() - int1;
            match callstack.pop().unwrap() {
                S(last_index) => {
                    let len = node_vec.len();
                    let final_index = int2 + int1 - 1 + len;
                    node_vec.get_mut(last_index).unwrap().get_children_mut().unwrap().push(len);
                    for i in 1..int1 {
                        let mut clone = node_vec.get(last_index).unwrap().clone();
                        let children = clone.get_children_mut().unwrap();
                        children.clear();
                        if i != int1 - 1 {
                            children.push(node_vec.len() + 1);
                        } else {
                            children.push(final_index);
                        }
                        node_vec.push(clone);
                    }
                    callstack.push(S(node_vec.len() - 1));

                    if possessive {
                        add_node(Node::StartAtomic {children: vec![]}, node_vec, callstack);
                    }

                    // --------------------Add Optional Nodes----------------------

                    let last_index = callstack.pop().unwrap().idx();
                    let len = node_vec.len();
                    node_vec.get_mut(last_index).unwrap().get_children_mut().unwrap().insert(0, len);
                    for _ in 0..int2 {
                        let mut clone = node_vec.get(last_index).unwrap().clone();
                        let children = clone.get_children_mut().unwrap();
                        children.clear();
                        children.push(node_vec.len() + 1);
                        children.lazy_dependent_insert(final_index, lazy);
                        node_vec.push(clone);
                    }
                    callstack.push(S(node_vec.len()));
                    if possessive {
                        node_vec.push(Node::EndAtomic {children: vec![]});
                    } else {
                        node_vec.push(Node::new_transition());
                    }
                }
                M {
                    mut last,
                    mut first,
                    mut after,
                    mut before,
                    ..
                } => {
                    let initial_first = first;
                    for _ in 0..int1.saturating_sub(1) {
                        let mut new_nodes = vec![];
                        let new_before_index = node_vec.len();
                        for i in first..node_vec.len() {
                            new_nodes.push(node_vec.get(i).unwrap().clone());
                        }
                        node_vec.get_mut(last).unwrap().get_children_mut().unwrap().push(new_before_index);
                        for i in 0..new_nodes.len() {
                            let len = new_nodes.len();
                            let node = new_nodes.get_mut(i).unwrap();
                            match node.get_children_mut() {
                                Some(c) => {
                                    for child in c {
                                        *child += len;
                                    }
                                }
                                None => continue,
                            }
                        }
                        last += new_nodes.len();
                        after += new_nodes.len();
                        before += new_nodes.len();
                        first += new_nodes.len();
                        node_vec.extend(new_nodes);
                    }
                    let final_idx = last + int2 * (node_vec.len() - first);
                    for _ in 0..int2 {
                        let mut new_nodes = vec![];
                        let new_before_index = node_vec.len();
                        for i in first..node_vec.len() {
                            new_nodes.push(node_vec.get(i).unwrap().clone());
                        }
                        node_vec.get_mut(last).unwrap().get_children_mut().unwrap().push(new_before_index);
                        for i in 0..new_nodes.len() {
                            let len = new_nodes.len();
                            let node = new_nodes.get_mut(i).unwrap();
                            match node.get_children_mut() {
                                Some(c) => {
                                    for child in c {
                                        *child += len;
                                    }
                                }
                                None => continue,
                            }
                        }
                        last += new_nodes.len();
                        after += new_nodes.len();
                        before += new_nodes.len();
                        first += new_nodes.len();
                        node_vec.extend(new_nodes);
                        node_vec.get_mut(first).unwrap().lazy_dependent_insert(final_idx, lazy);
                    }
                    callstack.push(M {
                        first: initial_first,
                        before,
                        after,
                        last,
                        to_link: last,
                    });
                }
            }
        }
    } else {
        let mut s1 = String::from("");
        for character in contents {
            s1.push(*character);
        }
        let to_repeat = s1.parse::<usize>().unwrap();
        match callstack.pop().unwrap() {
            S(last_index) => {
                let len = node_vec.len();
                node_vec.get_mut(last_index).unwrap().get_children_mut().unwrap().push(len);
                for i in 1..to_repeat {
                    let mut clone = node_vec.get(last_index).unwrap().clone();
                    let children = clone.get_children_mut().unwrap();
                    children.clear();
                    if i != to_repeat - 1 {
                        children.push(node_vec.len() + 1);
                    }
                    node_vec.push(clone);
                }
                callstack.push(S(node_vec.len() - 1));
            }
            M {
                mut before,
                mut after,
                first,
                mut last,
                ..
            } => {
                for _ in 0..to_repeat {
                    let mut new_nodes = vec![];
                    let new_before_index = node_vec.len();
                    for i in first..node_vec.len() {
                        new_nodes.push(node_vec.get(i).unwrap().clone());
                    }
                    node_vec.get_mut(last).unwrap().get_children_mut().unwrap().push(new_before_index);
                    for i in 0..new_nodes.len() {
                        let len = new_nodes.len();
                        let node = new_nodes.get_mut(i).unwrap();
                        match node.get_children_mut() {
                            Some(c) => {
                                for child in c {
                                    *child += len;
                                }
                            }
                            None => continue,
                        }
                    }
                    last += new_nodes.len();
                    after += new_nodes.len();
                    before += new_nodes.len();
                    node_vec.extend(new_nodes);
                }
                callstack.push(M {
                    first,
                    before,
                    after,
                    last,
                    to_link: last,
                });
            }
        }
    }
    if possessive {
        if let M { first, last, .. } = callstack.last().unwrap() {
            node_vec[*first] = Node::StartAtomic {
                children: node_vec[*first].get_children_mut().unwrap().clone(),
            };
            node_vec[*last] = Node::EndAtomic {
                children: node_vec[*last].get_children_mut().unwrap().clone(),
            };
        }
    }
}

fn get_enclosing_brackets_to_repeat(string: &[char], mut index: usize) -> Vec<char> {
    let mut s = vec![];
    if string[index] == ')' {
        let mut count = 0;
        loop {
            let character = string[index];
            s.push(character);
            if index == 0 {
                break;
            }
            if string[index] == '(' && !check_if_escaped(string, index) {
                count -= 1;
            }
            if string[index] == ')' && !check_if_escaped(string, index) {
                count += 1;
            }
            if count == 0 {
                break;
            }
            index -= 1;
        }
    } else if string[index] == ']' {
        loop {
            let character = string[index];
            s.push(character);
            if index == 0 {
                break;
            }
            if string[index] == '[' && !check_if_escaped(string, index) {
                break;
            }
            index -= 1;
        }
    } else {
        println!("{}", string[index]);
        panic!();
    }
    return s;
}

pub(crate) fn check_if_escaped(string: &[char], index: usize) -> bool {
    if index == 0 {
        return false;
    }
    if string[index - 1] == BACKSLASH {
        if index == 1 {
            return true;
        } else {
            if check_if_escaped(string, index - 1) {
                return false;
            }
            return true;
        }
    }
    return false;
}

pub(crate) fn add_node(node: Node, node_vec: &mut Vec<Node>, callstack: &mut Vec<ParseToken>) {
    let len = node_vec.len();
    node_vec.push(node);
    if let Some(to_connect) = callstack.pop() {
        let n2 = node_vec.get_mut(to_connect.idx()).unwrap();
        n2.push_child(len);
        // println!("Added Node: {:?} and linked to {}", node_vec.last().unwrap(), to_connect.idx());
    }
    callstack.push(ParseToken::S(len));
}

pub(crate) fn add_character(c: char, node_vec: &mut Vec<Node>, callstack: &mut Vec<ParseToken>) {
    add_node(Node::new_from_char(c), node_vec, callstack)
}
