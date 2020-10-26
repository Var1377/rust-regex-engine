use super::{
    compiled_node::*,
    constants::*,
    node::{Node, NodeMap},
    regex::Regex,
    utils::*,
};

impl Regex {
    pub fn new(mut regex: &str) -> Self {
        let mut r = Self::default();
        r.expr = regex.to_string();
        r.parse_expression();
        // r.compile();
        return r;
    }

    fn parse_expression(&mut self) {
        fn add_node(node: Node, map: &mut NodeMap, callstack: &mut Vec<usize>, map_index: &mut usize, chars: &Vec<char>, char_index: &usize) {
            if char_index > &0 {
                let lookback = chars[char_index - 1];
                match lookback {
                    '(' | '|' => callstack.push(callstack.last().unwrap().clone()),
                    _ => {}
                }
            }
            map.insert(map_index.clone(), node);
            if let Some(to_connect) = callstack.pop() {
                let mut node = map.get_mut(&to_connect).unwrap();
                match node {
                    Node::Inclusive { ref mut children, .. }
                    | Node::Exclusive { ref mut children, .. }
                    | Node::Transition { ref mut children, .. }
                    | Node::BeginningOfLine { ref mut children }
                    | Node::EndOfLine { ref mut children }
                    | Node::MatchOne { ref mut children, .. }
                    | Node::MatchAll { ref mut children }
                    | Node::NotMatchOne { ref mut children, .. } => {
                        children.push(map_index.clone());
                    }
                    Node::End => panic!("change transition function failed"),
                }
            }
            callstack.push(map_index.clone());
            *map_index += 1;
        }

        fn add_character(c: char, map: &mut NodeMap, callstack: &mut Vec<usize>, map_index: &mut usize, chars: &Vec<char>, char_index: &usize) {
            add_node(Node::new_from_char(c, false), map, callstack, map_index, chars, char_index)
        }

        fn parse(
            mut map: NodeMap,
            mut string: Vec<char>,
            mut map_index: usize,
            mut callstack: &mut Vec<usize>,
            mut upcoming_transition_stack: &mut Vec<usize>,
            mut string_index: usize,
        ) -> (NodeMap, usize, usize) {
            let mut escaped = false;
            let mut collecting_square_bracket_expr = false;
            let mut collecting_curly_brackets = false;
            let mut current_range = vec![];
            let mut current_curly = vec![];
            while string_index < string.len() {
                // println!("{:?}", callstack);
                let character = string[string_index];
                if collecting_curly_brackets {
                    if character == '}'{
                        string.remove(string_index);
                        parse_curly_brackets(&current_curly, &mut string, &mut string_index);
                        current_curly = vec![];
                        collecting_curly_brackets = false;
                        string_index -= 1;
                    } else {
                        current_curly.push(string.remove(string_index));
                        string_index -= 1;
                    }   
                }
                else if collecting_square_bracket_expr {
                    if character == ']' {
                        if string[string_index - 1] != BACKSLASH {
                            collecting_square_bracket_expr = false;
                            current_range = parse_square_brackets(current_range, &mut map_index, &mut map, &mut callstack);
                        } else {
                            current_range.push(string[string_index - 1]);
                        }
                    } else {
                        current_range.push(character);
                    }
                } else if escaped {
                    match character {
                        'n' => add_character('\n', &mut map, &mut callstack, &mut map_index, &string, &string_index),
                        'd' => add_node(
                            Node::new_from_chars(DIGITS.to_vec(), false),
                            &mut map,
                            &mut callstack,
                            &mut map_index,
                            &string,
                            &string_index,
                        ),
                        'D' => add_node(
                            Node::new_from_chars(DIGITS.to_vec(), true),
                            &mut map,
                            &mut callstack,
                            &mut map_index,
                            &string,
                            &string_index,
                        ),
                        's' => add_node(
                            Node::new_from_chars(WHITESPACE.to_vec(), false),
                            &mut map,
                            &mut callstack,
                            &mut map_index,
                            &string,
                            &string_index,
                        ),
                        'S' => add_node(
                            Node::new_from_chars(WHITESPACE.to_vec(), true),
                            &mut map,
                            &mut callstack,
                            &mut map_index,
                            &string,
                            &string_index,
                        ),
                        'w' => {
                            let mut vec = DIGITS.to_vec();
                            vec.extend(UPPERCASE);
                            vec.extend(LOWERCASE);
                            vec.push('_');
                            add_node(
                                Node::new_from_chars(vec, false),
                                &mut map,
                                &mut callstack,
                                &mut map_index,
                                &string,
                                &string_index,
                            );
                        }
                        'W' => {
                            let mut vec = DIGITS.to_vec();
                            vec.extend(UPPERCASE);
                            vec.extend(LOWERCASE);
                            vec.push('_');
                            add_node(
                                Node::new_from_chars(vec, true),
                                &mut map,
                                &mut callstack,
                                &mut map_index,
                                &string,
                                &string_index,
                            );
                        }
                        _ => {
                            add_character(character, &mut map, &mut callstack, &mut map_index, &string, &string_index);
                        }
                    };
                    escaped = false;
                } else {
                    match character {
                        BACKSLASH => {
                            escaped = true;
                        }
                        '(' => {
                            let before_index: usize = map_index.clone();
                            let mut before = Node::new_transition();
                            map_index += 1;
                            let after_index: usize = map_index.clone();
                            let mut after = Node::new_transition();
                            map_index += 1;
                            upcoming_transition_stack.push(after_index);
                            let old = callstack.pop().unwrap();
                            let mut to_connect = map.get(&old).unwrap().clone();
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
                            map.insert(old.clone(), to_connect);
                            map.insert(before_index.clone(), before);
                            map.insert(after_index.clone(), after);
                            callstack.push(before_index);
                        }
                        ')' => {
                            // println!("Before ) Operator {:?}", callstack);
                            let after_index = upcoming_transition_stack.last().unwrap();
                            let current_node_index = callstack.last().unwrap();
                            let mut current_node = map.get_mut(current_node_index).unwrap();
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
                            let node = map.get(to_connect).unwrap();
                            let mut new_node = node.clone();
                            match new_node {
                                Node::Transition { ref mut children, .. } => {
                                    children.push(map_index);
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
                            let current_node_index = callstack.last().unwrap();
                            let mut current_node = map.get(current_node_index).unwrap().clone();
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
                            map.insert(current_node_index.clone(), current_node);
                            callstack.pop();
                            // println!("After | Operator {:?}", callstack);
                        }
                        '+' => {
                            if previous_char_is_closing_bracket(&string_index, &string) {
                                let node_index = callstack.last().unwrap();
                                let mut after = map.get(&node_index).unwrap().clone();
                                let before_index = node_index - 1;
                                match after {
                                    Node::Transition { ref mut children, .. } => {
                                        children.push(before_index);
                                    }
                                    _ => panic!("Addition Compile error"),
                                }
                                map.insert(node_index.clone(), after);
                            } else {
                                let x = callstack.last().unwrap();
                                let mut node = map.get(x).unwrap().clone();
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
                                map.insert(x.clone(), node);
                            }
                        }
                        '*' => {
                            if previous_char_is_closing_bracket(&string_index, &string) {
                                let node_index = callstack.pop().unwrap();
                                let mut after = map.get_mut(&node_index).unwrap();
                                let before_index = node_index - 1;
                                match after {
                                    Node::Transition { ref mut children, .. } => {
                                        children.push(before_index);
                                    }
                                    _ => panic!("Addition Compile error"),
                                }
                                callstack.push(before_index);
                            } else {
                                let node_index = callstack.pop().unwrap();
                                let mut node = map.get(&node_index).unwrap().clone();
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
                                        children.push(node_index);
                                        map.insert(node_index + 1, node);
                                    }
                                    Node::End => panic!("Something went wrong here"),
                                };
                                match new_transition {
                                    Node::Transition { ref mut children, .. } => {
                                        children.push(node_index + 1);
                                    }
                                    _ => panic!("Something went wrong here"),
                                }
                                map.insert(node_index.clone(), new_transition);
                                callstack.push(node_index.clone());
                                map_index += 1;
                            }
                        }
                        '^' => add_node(
                            Node::new_start_of_line(),
                            &mut map,
                            &mut callstack,
                            &mut map_index,
                            &string,
                            &string_index,
                        ),
                        '$' => add_node(Node::new_end_of_line(), &mut map, &mut callstack, &mut map_index, &string, &string_index),
                        '.' => add_node(Node::new_match_all(), &mut map, &mut callstack, &mut map_index, &string, &string_index),
                        '?' => {
                            let mut q = |brackets: bool| {
                                if brackets {
                                let len = callstack.len();
                                let to_connect = callstack[len - 2];
                                let to_connect2 = callstack.last().unwrap();
                                let node = map.get_mut(&to_connect).unwrap();
                                match node {
                                    Node::Inclusive { ref mut children, .. }
                                    | Node::Exclusive { ref mut children, .. }
                                    | Node::Transition { ref mut children, .. }
                                    | Node::BeginningOfLine { ref mut children }
                                    | Node::EndOfLine { ref mut children }
                                    | Node::MatchOne { ref mut children, .. }
                                    | Node::MatchAll { ref mut children }
                                    | Node::NotMatchOne { ref mut children, .. } => {
                                        children.push(map_index);
                                    }
                                    Node::End => panic!(),
                                }
                                let node = map.get_mut(to_connect2).unwrap();
                                match node {
                                    Node::Inclusive { ref mut children, .. }
                                    | Node::Exclusive { ref mut children, .. }
                                    | Node::Transition { ref mut children, .. }
                                    | Node::BeginningOfLine { ref mut children }
                                    | Node::EndOfLine { ref mut children }
                                    | Node::MatchOne { ref mut children, .. }
                                    | Node::MatchAll { ref mut children }
                                    | Node::NotMatchOne { ref mut children, .. } => {
                                        children.push(map_index);
                                    }
                                    Node::End => panic!(),
                                };
                                map.insert(map_index, Node::new_transition());
                                callstack.push(map_index);
                                map_index += 1;
                            } else {
                                let mut new_transition1 = Node::new_transition();
                                let mut new_transition2 = Node::new_transition();
                                let old = callstack.pop().unwrap();
                                let mut old_node = map.get(&old).unwrap().clone();
                                match new_transition1 {
                                    Node::Transition {ref mut children } => {
                                        children.push(map_index);
                                        children.push(map_index + 2);
                                    }
                                    _ => panic!()
                                }
                                map.insert(old, new_transition1);
                                match old_node {
                                    Node::Inclusive { ref mut children, .. }
                                    | Node::Exclusive { ref mut children, .. }
                                    | Node::Transition { ref mut children, .. }
                                    | Node::BeginningOfLine { ref mut children }
                                    | Node::EndOfLine { ref mut children }
                                    | Node::MatchOne { ref mut children, .. }
                                    | Node::MatchAll { ref mut children }
                                    | Node::NotMatchOne { ref mut children, .. } => {
                                        children.push(map_index + 1);
                                    }
                                    Node::End => panic!(),
                                }
                                map.insert(map_index, old_node);
                                map_index += 1;
                                map.insert(map_index, new_transition2);
                                callstack.push(map_index);
                                map_index += 1;
                            }
                            };
                            if previous_char_is_closing_bracket(&string_index, &string) {
                                q(true)
                            } else {
                                q(false);
                            }
                        },
                        '{' => {
                            collecting_curly_brackets = true;
                            string.remove(string_index);
                            string_index -= 1;
                        },
                        _ =>  add_character(character, &mut map, callstack, &mut map_index, &string, &string_index),
                    }
                }
                string_index += 1;
            }
            // println!("Final {:?}", callstack);
            let index = callstack.last().unwrap();
            let mut x = map.get(index).unwrap().clone();
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
                    map.insert(index.clone(), x);
                }
            };
            map_index += 1;
            return (map, map_index, string_index);
        }
        // println!("compiling");
        let mut map = NodeMap::default();
        map.insert(0, Node::new_transition());
        map.insert(1, Node::End);
        let (new_tree, _, _) = parse(map, str_to_char_vec(self.expr.as_str()), 2, &mut vec![0, 0], &mut vec![1], 0);
        self.tree = Some(new_tree);
    }

    fn compile(&mut self) {
        let mut tree = self.tree.clone().unwrap();
        // self.tree = None;
        let root = CompiledNode::map_to_compiled_node_tree(&mut tree);
        self.compiled_root = Some(root);
    }
}
