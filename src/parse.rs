use super::{constants::*, node::*, regex::Regex, utils::*};

impl Regex {
    pub(crate) fn parse_expression(&mut self) {
        fn add_node(node: Node, node_vec: &mut Vec<Node>, callstack: &mut Vec<usize>, chars: &Vec<char>, char_index: &usize) {
            if char_index > &0 {
                let lookback = chars[char_index - 1];
                match lookback {
                    '(' | '|' => callstack.push(callstack.last().unwrap().clone()),
                    _ => {}
                }
            }
            node_vec.push(node);
            let len = node_vec.len() - 1;
            if let Some(to_connect) = callstack.pop() {
                let node = node_vec.get_mut(to_connect).unwrap();
                match node {
                    Node::Inclusive { ref mut children, .. }
                    | Node::Exclusive { ref mut children, .. }
                    | Node::Transition { ref mut children, .. }
                    | Node::BeginningOfLine { ref mut children }
                    | Node::EndOfLine { ref mut children }
                    | Node::MatchOne { ref mut children, .. }
                    | Node::MatchAll { ref mut children }
                    | Node::NotMatchOne { ref mut children, .. } => {
                        children.push(len);
                    }
                    Node::End => panic!("change transition function failed"),
                }
            }
            callstack.push(len);
        }

        fn add_character(c: char, node_vec: &mut Vec<Node>, callstack: &mut Vec<usize>, chars: &Vec<char>, char_index: &usize) {
            add_node(Node::new_from_char(c, false), node_vec, callstack, chars, char_index)
        }

        fn parse(
            mut node_vec: Vec<Node>,
            mut string: Vec<char>,
            mut callstack: &mut Vec<usize>,
            upcoming_transition_stack: &mut Vec<usize>,
            mut string_index: usize,
        ) -> (Vec<Node>, usize) {
            let mut escaped = false;
            let mut collecting_square_bracket_expr = false;
            let mut collecting_curly_brackets = false;
            let mut current_range = vec![];
            let mut current_curly = vec![];
            while string_index < string.len() {
                // println!("{:?}", callstack);
                let character = string[string_index];
                if collecting_curly_brackets {
                    if character == '}' {
                        string.remove(string_index);
                        parse_curly_brackets(&current_curly, &mut string, &mut string_index);
                        current_curly = vec![];
                        collecting_curly_brackets = false;
                        string_index -= 1;
                    } else {
                        current_curly.push(string.remove(string_index));
                        string_index -= 1;
                    }
                } else if collecting_square_bracket_expr {
                    if character == ']' {
                        if !check_if_escaped(&string, string_index) {
                            collecting_square_bracket_expr = false;
                            current_range = parse_square_brackets(current_range, &mut node_vec, &mut callstack);
                        } else {
                            current_range.push(string[string_index - 1]);
                        }
                    } else {
                        current_range.push(character);
                    }
                } else if escaped {
                    match character {
                        'n' => add_character('\n', &mut node_vec, &mut callstack, &string, &string_index),
                        'd' => add_node(
                            Node::new_from_chars(DIGITS.to_vec(), false),
                            &mut node_vec,
                            &mut callstack,
                            &string,
                            &string_index,
                        ),
                        'D' => add_node(
                            Node::new_from_chars(DIGITS.to_vec(), true),
                            &mut node_vec,
                            &mut callstack,
                            &string,
                            &string_index,
                        ),
                        's' => add_node(
                            Node::new_from_chars(WHITESPACE.to_vec(), false),
                            &mut node_vec,
                            &mut callstack,
                            &string,
                            &string_index,
                        ),
                        'S' => add_node(
                            Node::new_from_chars(WHITESPACE.to_vec(), true),
                            &mut node_vec,
                            &mut callstack,
                            &string,
                            &string_index,
                        ),
                        'w' => {
                            let mut vec = DIGITS.to_vec();
                            vec.extend(UPPERCASE);
                            vec.extend(LOWERCASE);
                            vec.push('_');
                            add_node(Node::new_from_chars(vec, false), &mut node_vec, &mut callstack, &string, &string_index);
                        }
                        'W' => {
                            let mut vec = DIGITS.to_vec();
                            vec.extend(UPPERCASE);
                            vec.extend(LOWERCASE);
                            vec.push('_');
                            add_node(Node::new_from_chars(vec, true), &mut node_vec, &mut callstack, &string, &string_index);
                        }
                        _ => {
                            add_character(character, &mut node_vec, &mut callstack, &string, &string_index);
                        }
                    };
                    escaped = false;
                } else {
                    match character {
                        BACKSLASH => {
                            escaped = true;
                        }
                        '(' => {
                            let before_index: usize = node_vec.len();
                            let before = Node::new_transition();
                            node_vec.push(before);
                            let after_index: usize = node_vec.len();
                            let after = Node::new_transition();
                            node_vec.push(after);
                            upcoming_transition_stack.push(after_index);
                            let old = callstack.pop().unwrap();
                            let to_connect = node_vec.get_mut(old).unwrap();
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
                                Node::End => panic!("aah"),
                            }
                            callstack.push(before_index);
                        }
                        ')' => {
                            // println!("Before ) Operator {:?}", callstack);
                            let after_index = upcoming_transition_stack.last().unwrap();
                            let current_last_node_index = callstack.last().unwrap();
                            let current_node = node_vec.get_mut(current_last_node_index.clone()).unwrap();
                            match current_node {
                                Node::Inclusive { ref mut children, .. }
                                | Node::Exclusive { ref mut children, .. }
                                | Node::Transition { ref mut children, .. }
                                | Node::BeginningOfLine { ref mut children }
                                | Node::EndOfLine { ref mut children }
                                | Node::MatchOne { ref mut children, .. }
                                | Node::MatchAll { ref mut children }
                                | Node::NotMatchOne { ref mut children, .. } => {
                                    children.push(after_index.clone());
                                }
                                Node::End => panic!("something went wrong"),
                            }
                            let to_connect = upcoming_transition_stack.last().unwrap();
                            let node = node_vec.get(to_connect.clone()).unwrap();
                            let mut new_node = node.clone();
                            match new_node {
                                Node::Transition { ref mut children, .. } => {
                                    children.push(node_vec.len() - 1);
                                }
                                _ => panic!("Expected transition node found something else"),
                            }
                            callstack.pop();
                            let upcoming = upcoming_transition_stack.pop().unwrap();
                            callstack.push(upcoming);
                            // println!("After ) Operator {:?}", callstack);
                        }
                        '[' => {
                            collecting_square_bracket_expr = true;
                        }
                        '|' => {
                            // println!("Before | Operator {:?}", callstack);
                            let after_index = upcoming_transition_stack.last().unwrap();
                            let current_last_node_index = callstack.last().unwrap();
                            let mut current_node = node_vec.get(current_last_node_index.clone()).unwrap().clone();
                            match &mut current_node {
                                Node::Inclusive { ref mut children, .. }
                                | Node::Exclusive { ref mut children, .. }
                                | Node::Transition { ref mut children, .. }
                                | Node::BeginningOfLine { ref mut children }
                                | Node::EndOfLine { ref mut children }
                                | Node::MatchOne { ref mut children, .. }
                                | Node::MatchAll { ref mut children }
                                | Node::NotMatchOne { ref mut children, .. } => {
                                    children.push(after_index.clone());
                                }
                                Node::End => panic!("something went wrong"),
                            }
                            node_vec[current_last_node_index.clone()] = current_node;
                            callstack.pop();
                            // println!("After | Operator {:?}", callstack);
                        }
                        '+' => {
                            if previous_char_is_closing_bracket(&string_index, &string) {
                                let last_node_index = callstack.last().unwrap();
                                let after = node_vec.get_mut(last_node_index.clone()).unwrap();
                                let before_index = last_node_index - 1;
                                match after {
                                    Node::Transition { ref mut children, .. } => {
                                        children.push(before_index);
                                    }
                                    _ => panic!("Addition Compile error"),
                                }
                                println!(
                                    " + Before: {}, callstack: {:?}, final index of vec: {}",
                                    before_index,
                                    callstack,
                                    node_vec.len() - 1
                                );
                            } else {
                                let x = callstack.last().unwrap().clone();
                                let node = node_vec.get_mut(x).unwrap();
                                match node {
                                    Node::Inclusive { ref mut children, .. }
                                    | Node::Exclusive { ref mut children, .. }
                                    | Node::Transition { ref mut children, .. }
                                    | Node::BeginningOfLine { ref mut children }
                                    | Node::EndOfLine { ref mut children }
                                    | Node::MatchOne { ref mut children, .. }
                                    | Node::MatchAll { ref mut children }
                                    | Node::NotMatchOne { ref mut children, .. } => {
                                        children.push(x.clone());
                                    }
                                    Node::End => panic!("Addition Compile error"),
                                }
                            }
                        }
                        '*' => {
                            if previous_char_is_closing_bracket(&string_index, &string) {
                                let last_node_index = callstack.pop().unwrap();
                                let after = node_vec.get_mut(last_node_index).unwrap();
                                let before_index = last_node_index - 1;
                                match after {
                                    Node::Transition { ref mut children, .. } => {
                                        children.push(before_index);
                                    }
                                    _ => panic!("Addition Compile error"),
                                }
                                callstack.push(before_index);
                            } else {
                                let last_node_index = callstack.pop().unwrap();
                                let mut node = node_vec.get(last_node_index).unwrap().clone();
                                let mut new_transition = Node::new_transition();
                                match node {
                                    Node::Inclusive { ref mut children, .. }
                                    | Node::Exclusive { ref mut children, .. }
                                    | Node::Transition { ref mut children, .. }
                                    | Node::BeginningOfLine { ref mut children }
                                    | Node::EndOfLine { ref mut children }
                                    | Node::MatchOne { ref mut children, .. }
                                    | Node::MatchAll { ref mut children }
                                    | Node::NotMatchOne { ref mut children, .. } => {
                                        children.push(last_node_index);
                                        node_vec.push(node);
                                    }
                                    Node::End => panic!("Something went wrong here"),
                                };
                                match new_transition {
                                    Node::Transition { ref mut children, .. } => {
                                        children.push(last_node_index + 1);
                                    }
                                    _ => panic!("Something went wrong here"),
                                }
                                node_vec[last_node_index] = new_transition;
                                callstack.push(last_node_index);
                            }
                        }
                        '^' => add_node(Node::new_start_of_line(), &mut node_vec, &mut callstack, &string, &string_index),
                        '$' => add_node(Node::new_end_of_line(), &mut node_vec, &mut callstack, &string, &string_index),
                        '.' => add_node(Node::new_match_all(), &mut node_vec, &mut callstack, &string, &string_index),
                        '?' => {
                            let mut q = |brackets: bool| {
                                if brackets {
                                    // println!("{:?}", callstack);
                                    let before_index = callstack.last().unwrap() - 1;
                                    let after_index = callstack.last().unwrap().clone();
                                    let before = node_vec.get_mut(before_index).unwrap();
                                    match before {
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
                                } else {
                                    let mut new_transition1 = Node::new_transition();
                                    let new_transition2 = Node::new_transition();
                                    let old = callstack.pop().unwrap();
                                    let mut old_node = node_vec.get(old).unwrap().clone();
                                    match new_transition1 {
                                        Node::Transition { ref mut children } => {
                                            children.push(node_vec.len());
                                            children.push(node_vec.len() + 1);
                                        }
                                        _ => panic!(),
                                    }
                                    node_vec[old] = new_transition1;
                                    match old_node {
                                        Node::Inclusive { ref mut children, .. }
                                        | Node::Exclusive { ref mut children, .. }
                                        | Node::Transition { ref mut children, .. }
                                        | Node::BeginningOfLine { ref mut children }
                                        | Node::EndOfLine { ref mut children }
                                        | Node::MatchOne { ref mut children, .. }
                                        | Node::MatchAll { ref mut children }
                                        | Node::NotMatchOne { ref mut children, .. } => {
                                            children.push(node_vec.len() + 1);
                                        }
                                        Node::End => panic!(),
                                    }
                                    node_vec.push(old_node);
                                    node_vec.push(new_transition2);
                                    callstack.push(node_vec.len() - 1);
                                }
                            };
                            if previous_char_is_closing_bracket(&string_index, &string) {
                                q(true)
                            } else {
                                q(false);
                            }
                        }
                        '{' => {
                            collecting_curly_brackets = true;
                            string.remove(string_index);
                            string_index -= 1;
                        }
                        _ => add_character(character, &mut node_vec, callstack, &string, &string_index),
                    }
                }
                string_index += 1;
            }
            // println!("Final {:?}", callstack);
            let index = callstack.last().unwrap().clone();
            let x = node_vec.get_mut(index).unwrap();
            match x {
                Node::End => {}
                Node::Exclusive { ref mut children, .. }
                | Node::Inclusive { ref mut children, .. }
                | Node::Transition { ref mut children, .. }
                | Node::MatchAll { ref mut children, .. }
                | Node::BeginningOfLine { ref mut children }
                | Node::EndOfLine { ref mut children }
                | Node::MatchOne { ref mut children, .. }
                | Node::NotMatchOne { ref mut children, .. } => {
                    children.push(1);
                }
            };
            println!("Node Vec: {:?}", node_vec);
            return (node_vec, string_index);
        }
        // println!("compiling");
        let mut node_vec = Vec::<Node>::default();
        node_vec.push(Node::new_transition());
        node_vec.push(Node::End);
        let (new_tree, _) = parse(node_vec, str_to_char_vec(&self.expr), &mut vec![0, 0], &mut vec![1], 0);
        self.node_vec = new_tree;
    }
}
