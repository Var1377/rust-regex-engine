use super::{constants::*, node::*};

#[inline]
pub fn str_to_char_vec(string: &str) -> Vec<char> {
    let mut vec = Vec::with_capacity(string.len());
    for c in string.bytes() {
        vec.push(c as char);
    }
    return vec;
}

#[inline]
pub fn parse_range(c1: char, c2: char, exclusive: bool) -> Node {
    let mut range = vec![];
    for cat in [LOWERCASE, UPPERCASE, DIGITS].iter() {
        if cat.contains(&c1) {
            let (i, j) = (cat.iter().position(|&x| x == c1).unwrap(), cat.iter().position(|&x| x == c2).unwrap());
            for x in i..=j {
                range.push(cat[x]);
            }
        }
    }
    return Node::new_from_chars(range, exclusive);
}

#[inline]
pub fn previous_char_is_closing_bracket(index: &usize, chars: &[char]) -> bool {
    if index < &2 {
        return false;
    } else {
        let lookback = chars[index-1];
        if lookback == ')'||lookback == ']' {
            let lookback2 = chars[index-2];
            if lookback2 == BACKSLASH {
                return false;
            }
            return true;
        } else {
            return false;
        }
    }
}

#[inline]
pub fn parse_range_character(c: char) -> Node {
    match c {
        'd' => return Node::new_from_chars(DIGITS.to_vec(), false),
        'D' => return Node::new_from_chars(DIGITS.to_vec(), true),
        'w' => {
            let mut vec = DIGITS.to_vec();
            vec.extend(UPPERCASE.to_vec());
            vec.extend(LOWERCASE.to_vec());
            vec.push('_');
            return Node::new_from_chars(vec, false);
        }
        'W' => {
            let mut vec = DIGITS.to_vec();
            vec.extend(UPPERCASE.to_vec());
            vec.extend(LOWERCASE.to_vec());
            vec.push('_');
            return Node::new_from_chars(vec, true);
        }
        's' => return Node::new_from_chars(WHITESPACE.to_vec(), false),
        'S' => return Node::new_from_chars(WHITESPACE.to_vec(), true),

        _ => panic!("Range character not supported"),
    };
}

pub fn parse_square_brackets(chars: Vec<char>, node_vec: &mut Vec::<Node>, callstack: &mut Vec<usize>) -> Vec::<char> {
    // println!("Square Expression: {:?}", chars);
    let before = Node::new_transition();
    let before_index = node_vec.len();
    node_vec.push(before);
    let after = Node::new_transition();
    let after_index = node_vec.len();
    node_vec.push(after);
    let mut exclusive = false;
    let mut nodes = Vec::<Node>::new();
    if chars[0] == '^' {
        exclusive = true;
    }
    let mut i: usize;
    if exclusive {
        i = 1;
    } else {
        i = 0;
    }
    let len = chars.len();
    let mut ranges = vec![];
    let mut rest_of_chars = vec![];
    while i < len {
        let character = chars[i];
        if character == '-' {
            if i != 0 && chars[i - 1] != BACKSLASH && i != len - 1 {
                ranges.push((chars[i - 1], chars[i + 1]));
            } else {
                rest_of_chars.push(character);
            }
        } else {
            if i == 0 {
                if len > 1 {
                    if chars[i + 1] == '-' {
                    } else {
                        rest_of_chars.push(character)
                    }
                } else {
                    rest_of_chars.push(character);
                }
            } else if i == len - 1 {
                if chars[i - 1] == '-' {
                } else {
                    rest_of_chars.push(character);
                }
            } else {
                if chars[i - 1] == '-' || chars[i + 1] == '-' {
                    if character != BACKSLASH {
                    } else {
                        rest_of_chars.push(character);
                    }
                } else {
                    rest_of_chars.push(character);
                }
            }
        }
        i += 1;
    }
    for range in ranges {
        let (c1, c2) = range;
        nodes.push(parse_range(c1, c2, exclusive));
    }
    let len = rest_of_chars.len();
    let mut escaped = false;
    i = 0;
    let mut final_range = vec![];
    while i < len {
        let character = rest_of_chars[i];
        if escaped {
            let lookback = rest_of_chars[i - 1];
            match lookback {
                BACKSLASH => match character {
                    'd' => {
                        if exclusive {
                            nodes.push(parse_range_character('D'));
                        } else {
                            nodes.push(parse_range_character('d'));
                        }
                    }
                    'D' => {
                        if exclusive {
                            nodes.push(parse_range_character('d'));
                        } else {
                            nodes.push(parse_range_character('D'));
                        }
                    }
                    'w' => {
                        if exclusive {
                            nodes.push(parse_range_character('W'));
                        } else {
                            nodes.push(parse_range_character('w'));
                        }
                    }
                    'W' => {
                        if exclusive {
                            nodes.push(parse_range_character('w'));
                        } else {
                            nodes.push(parse_range_character('W'));
                        }
                    }
                    's' => {
                        if exclusive {
                            nodes.push(parse_range_character('S'));
                        } else {
                            nodes.push(parse_range_character('s'));
                        }
                    }
                    'S' => {
                        if exclusive {
                            nodes.push(parse_range_character('s'));
                        } else {
                            nodes.push(parse_range_character('S'));
                        }
                    }
                    _ => nodes.push(Node::new_from_char(character, exclusive)),
                },
                _ => (),
            }
            escaped = false;
        } else {
            match character {
                BACKSLASH => escaped = true,
                _ => {
                    final_range.push(character);
                }
            };
        }
        i += 1;
    }
    nodes.push(Node::new_from_chars(final_range, exclusive));
    for mut node in nodes {
        let node_vec_len = node_vec.len();
        match node_vec.get_mut(before_index).unwrap() {
            Node::Transition { ref mut children, .. } => {
                children.push(node_vec_len);
            }
            _ => panic!(),
        }
        match node {
            Node::Inclusive { ref mut children, .. }
            | Node::Exclusive { ref mut children, .. }
            | Node::Transition { ref mut children, .. }
            | Node::BeginningOfLine { ref mut children }
            | Node::EndOfLine { ref mut children }
            | Node::MatchOne { ref mut children, .. }
            | Node::MatchAll { ref mut children }
            | Node::NotMatchOne { ref mut children, .. } => {
                children.push(after_index);
            }
            Node::End => panic!(),
        }
        node_vec.push(node);
    }
    let to_connect = callstack.pop().unwrap();
    let to_connect = node_vec.get_mut(to_connect).unwrap();
    match to_connect {
        Node::Inclusive { ref mut children, .. }
        | Node::Exclusive { ref mut children, .. }
        | Node::Transition { ref mut children, .. }
        | Node::BeginningOfLine { ref mut children }
        | Node::EndOfLine { ref mut children }
        | Node::MatchOne { ref mut children, .. }
        | Node::MatchAll { ref mut children }
        | Node::NotMatchOne { ref mut children, .. } => {
            children.push(before_index);
        }
        Node::End => panic!(),
    }
    callstack.push(after_index);
    return vec![];
}

pub fn parse_curly_brackets(contents: &Vec::<char>, string: &mut Vec::<char>, string_index: &usize) {
    println!("{:?}", contents);
    let pos_of_comma = contents.iter().position(|c| *c==',');
    if let Some(p) = pos_of_comma {
        if p == contents.len() -1 {
            let mut s1 = String::from("");
            for i in 0..p {
                s1.push(contents[i]);
            }
            let to_repeat = s1.parse::<usize>().unwrap();
            if previous_char_is_closing_bracket(string_index, string) {

            } else {
                string.insert(*string_index, '+');
                let previous_character = string[*string_index-1];
                for _ in 0..to_repeat-1 {
                    string.insert(*string_index, previous_character);
                }
            }
            // Ends in no number, up to x -> inf many times
        } else {
            // Ends in a number, x-> y times
        }
    } else {
        let mut s1 = String::from("");
        for character in contents {
            s1.push(*character);
        }
        println!("{}", s1);
        let to_repeat = s1.parse::<usize>().unwrap();
        if previous_char_is_closing_bracket(string_index, string) {

        } else {
            println!("{}", string_index);
            let previous_character = string[*string_index-1];
            for _ in 0..to_repeat-1 {
                string.insert(*string_index, previous_character);
            }
        }
    }
    println!("{:?}", string);
}