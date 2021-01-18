use super::{compiled_node::OptionBool, constants::*, nfa::*};

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

// pub(crate) fn parse_range(c1: char, c2: char, exclusive: bool) -> Node {
//     let mut range = vec![];
//     for cat in [LOWERCASE, UPPERCASE, DIGITS].iter() {
//         if cat.contains(&c1) {
//             let (i, j) = (cat.iter().position(|&x| x == c1).unwrap(), cat.iter().position(|&x| x == c2).unwrap());
//             for x in i..=j {
//                 range.push(cat[x]);
//             }
//         }
//     }
//     return Node::new_from_chars(range, exclusive);
// }

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
        'd' => return Node::new_from_chars(DIGITS.to_vec(), false),
        'D' => return Node::new_from_chars(DIGITS.to_vec(), true),
        'w' => {
            return Node::new_from_chars(W.to_vec(), false);
        }
        'W' => {
            return Node::new_from_chars(W.to_vec(), true);
        }
        's' => return Node::new_from_chars(WHITESPACE.to_vec(), false),
        'S' => return Node::new_from_chars(WHITESPACE.to_vec(), true),
        'b' => return Node::WordBoundary { children: vec![] },
        'B' => return Node::NotWordBoundary { children: vec![] },
        _ => panic!("Range character not supported"),
    };
}

pub(crate) fn parse_square_brackets(chars: &mut Vec<char>, node_vec: &mut Vec<Node>, callstack: &mut Vec<usize>) -> bool {
    // println!("Square Expression: {:?}", chars);
    if chars.len() == 0 {
        return false;
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
            return false;
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
            return false;
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
            return false;
        }

        // Have to have this in one node due to the properties of being exclusive

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

        return false;
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
            return false;
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
            return false;
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
            return false;
        } else {
            let before_index = node_vec.len();
            add_node(Node::new_transition(), node_vec, callstack);
            // let before = node_vec.get_mut(before_index);
            let after = Node::new_transition();
            let after_index = node_vec.len();
            node_vec.push(after);
            callstack.pop();
            callstack.push(after_index);
            let range_node = Node::InclusiveRange {
                characters: ranges,
                children: vec![after_index],
            };
            let character_node = Node::Inclusive {
                characters: match_characters,
                children: vec![after_index],
            };
            node_vec.push(range_node);
            node_vec.push(character_node);

            let before = node_vec.get_mut(before_index).unwrap().get_children_mut().unwrap();
            before.push(after_index + 1);
            before.push(after_index + 2);

            return true;
        }
    }
}

pub(crate) fn parse_curly_brackets(contents: &Vec<char>, closing_bracket: bool, node_vec: &mut Vec<Node>, callstack: &mut Vec<usize>, _lazy: bool) {
    let pos_of_comma = contents.iter().position(|c| *c == ',');
    if let Some(p) = pos_of_comma {
        if p == contents.len() - 1 {
            let mut s1 = String::new();
            for i in 0..p {
                s1.push(contents[i]);
            }
            let to_repeat = s1.parse::<usize>().unwrap() - 1;
            if closing_bracket {
                for _ in 0..to_repeat {
                    let to_connect = callstack.pop().unwrap();
                    let before_index = to_connect - 1;
                    let mut new_nodes = vec![];
                    let new_before_index = node_vec.len();
                    for i in before_index..node_vec.len() {
                        new_nodes.push(node_vec.get(i).unwrap().clone());
                    }
                    node_vec.get_mut(to_connect).unwrap().get_children_mut().unwrap().push(new_before_index);
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
                    callstack.push(node_vec.len() + 1);
                    node_vec.extend(new_nodes);
                }
                let last_node_index = callstack.last().unwrap();
                let after = node_vec.get_mut(*last_node_index).unwrap();
                let before_index = last_node_index - 1;
                after.get_transition_children_mut().push(before_index);
            } else {
                let last_index = callstack.pop().unwrap();
                let len = node_vec.len();
                node_vec.get_mut(last_index).unwrap().get_children_mut().unwrap().push(len);
                for i in 0..to_repeat {
                    let mut clone = node_vec.get(last_index).unwrap().clone();
                    let children = clone.get_children_mut().unwrap();
                    children.clear();
                    if i != to_repeat - 1 {
                        children.push(node_vec.len() + 1);
                    } else {
                        children.push(node_vec.len());
                    }
                    node_vec.push(clone);
                }
                callstack.push(node_vec.len() - 1);
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
            if closing_bracket {
                for _ in 0..int1 - 1 {
                    let to_connect = callstack.pop().unwrap();
                    let before_index = to_connect - 1;
                    let mut new_nodes = vec![];
                    let new_before_index = node_vec.len();
                    for i in before_index..node_vec.len() {
                        new_nodes.push(node_vec.get(i).unwrap().clone());
                    }
                    node_vec.get_mut(to_connect).unwrap().get_children_mut().unwrap().push(new_before_index);
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
                    callstack.push(node_vec.len() + 1);
                    node_vec.extend(new_nodes);
                }
                for _ in 0..int2 {
                    let to_connect = callstack.pop().unwrap();
                    let before_index = to_connect - 1;
                    let mut new_nodes = vec![];
                    let new_before_index = node_vec.len();
                    for i in before_index..node_vec.len() {
                        new_nodes.push(node_vec.get(i).unwrap().clone());
                    }
                    node_vec.get_mut(to_connect).unwrap().get_children_mut().unwrap().push(new_before_index);
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
                    callstack.push(node_vec.len() + 1);
                    node_vec.extend(new_nodes);
                    let before_index = callstack.last().unwrap() - 1;
                    let after_index = callstack.last().unwrap().clone();
                    let before = node_vec.get_mut(before_index).unwrap();
                    before.push_child(after_index);
                }
            } else {
                let last_index = callstack.pop().unwrap();
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
                callstack.push(node_vec.len() - 1);

                // --------------------Add Optional Nodes----------------------

                let last_index = callstack.pop().unwrap();
                let len = node_vec.len();
                node_vec.get_mut(last_index).unwrap().get_children_mut().unwrap().insert(0, len);
                for _ in 0..int2 {
                    let mut clone = node_vec.get(last_index).unwrap().clone();
                    let children = clone.get_children_mut().unwrap();
                    children.clear();
                    children.push(node_vec.len() + 1);
                    children.push(final_index);
                    node_vec.push(clone);
                }
                callstack.push(node_vec.len());
                node_vec.push(Node::new_transition());
            }
            // Ends in a number, x-> y times
        }
    } else {
        let mut s1 = String::from("");
        for character in contents {
            s1.push(*character);
        }
        let to_repeat = s1.parse::<usize>().unwrap();
        if closing_bracket {
            for _ in 0..to_repeat - 1 {
                let to_connect = callstack.pop().unwrap();
                let before_index = to_connect - 1;
                let mut new_nodes = vec![];
                let new_before_index = node_vec.len();
                for i in before_index..node_vec.len() {
                    new_nodes.push(node_vec.get(i).unwrap().clone());
                }
                node_vec.get_mut(to_connect).unwrap().get_children_mut().unwrap().push(new_before_index);
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
                callstack.push(node_vec.len() + 1);
                node_vec.extend(new_nodes);
            }
        } else {
            let last_index = callstack.pop().unwrap();
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
            callstack.push(node_vec.len() - 1);
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

pub(crate) fn add_node(node: Node, node_vec: &mut Vec<Node>, callstack: &mut Vec<usize>) {
    node_vec.push(node);
    let len = node_vec.len() - 1;
    if let Some(to_connect) = callstack.pop() {
        let node = node_vec.get_mut(to_connect).unwrap();
        node.push_child(len);
    }
    callstack.push(len);
}

pub(crate) fn add_character(c: char, node_vec: &mut Vec<Node>, callstack: &mut Vec<usize>) {
    add_node(Node::new_from_char(c), node_vec, callstack)
}
