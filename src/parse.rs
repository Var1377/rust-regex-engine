use super::{
    constants::*,
    node::{Node, NodeMap},
    regex::Regex,
    utils::*,
    compiled_node::*,
};

impl Regex {
    pub fn new(mut regex: String) -> Self {
        let mut r = Self::default();
        r.expr = regex;
        r.parse_expression();
        // r.compile();
        return r;
    }

    fn parse_expression(&mut self) {
        fn add_character(c: char, map: &mut NodeMap, callstack: &mut Vec<usize>, map_index: &mut usize, chars: &Vec<char>, char_index: &usize) {
            // println!("{:?}", map);
            // println!("Before: {:?}", callstack);
            if char_index > &0 {
                let lookback = chars.get(char_index - 1).unwrap();
                match lookback {
                    '(' | '|' => callstack.push(callstack.last().unwrap().clone()),
                    _ => {}
                }
            }
            map.insert(map_index.clone(), Node::new_from_char(c));
            if let Some(to_connect) = callstack.pop() {
                let mut node = map.get(&to_connect).unwrap().clone();
                match node {
                    Node::Inclusive { ref mut children, .. }
                    | Node::Exclusive { ref mut children, .. }
                    | Node::Transition { ref mut children, .. }
                    | Node::BeginningOfLine { ref mut children }
                    | Node::EndOfLine { ref mut children }
                    | Node::MatchOne { ref mut children, .. }
                    | Node::MatchAll { ref mut children } => {
                        children.push(map_index.clone());
                        map.insert(to_connect, node);
                    }
                    Node::End => panic!("change transition function failed"),
                }
            }
            callstack.push(map_index.clone());
            *map_index += 1;
            // println!("After: {:?}", callstack);
        }

        fn parse_square_brackets(chars: Vec<char>, map_index: &mut usize, node_map: &mut NodeMap) -> Vec<char> {
            // let mut before = Node::new_transition();
            // let before_index = map_index.clone();
            // *map_index += 1;
            // let mut after = Node::new_transition();
            // let after_index = map_index.clone();
            // *map_index += 1;
            // let mut exclusive = false;
            // let mut nodes = Vec::<Node>::new();
            // if chars[0] == '^' {
            //     exclusive = true;
            // }
            // let mut i: usize;
            // if exclusive {
            //     i = 1;
            // } else {
            //     i = 0;
            // }
            // let len = chars.len();
            // let mut ranges = vec![];
            // let mut rest_of_chars = vec![];
            // while i < len {
            //     let character = chars[i];
            //     if character == '-' {
            //         if i != 0 && chars[i - 1] != BACKSLASH {
            //             ranges.push((chars[i - 1], chars[i + 1]));
            //         } else {
            //             rest_of_chars.push(character);
            //         }
            //     } else {
            //         rest_of_chars.push(character);
            //     }
            //     i += 1;
            // }
            // for range in ranges {
            //     let (c1, c2) = range;
            //     nodes.push(parse_range(c1, c2, exclusive));
            // }
            return vec![];
        }

        fn parse(
            mut map: NodeMap,
            string: Vec<char>,
            mut map_index: usize,
            mut callstack: &mut Vec<usize>,
            mut upcoming_transition_stack: &mut Vec<usize>,
            mut string_index: usize,
        ) -> (NodeMap, usize, usize) {
            let mut looking_back = false;
            let mut collecting_range = false;
            let len = string.len();
            let mut current_range = vec![];
            while string_index < len {
                let character = string[string_index];
                if collecting_range {
                    if character == ']' {
                        if string[string_index - 1] != BACKSLASH {
                            collecting_range = false;
                            current_range = parse_square_brackets(current_range, &mut map_index, &mut map);
                        } else {
                            current_range.push(string[string_index - 1]);
                        }
                    } else {
                        current_range.push(character);
                    }
                } else if looking_back {
                    let lookback = string[string_index - 1];
                    match lookback {
                        BACKSLASH => match character {
                            'n' => add_character('\n', &mut map, &mut callstack, &mut map_index, &string, &string_index),
                            _ => {
                                add_character(character, &mut map, &mut callstack, &mut map_index, &string, &string_index);
                            }
                        },
                        _ => panic!("Something has gone wrong here"),
                    }
                } else {
                    match character {
                        BACKSLASH => {
                            looking_back = true;
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
                                | Node::MatchAll { ref mut children } => {
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
                            let mut current_node = map.get(current_node_index).unwrap().clone();
                            match &mut current_node {
                                Node::Inclusive { ref mut children, .. }
                                | Node::Exclusive { ref mut children, .. }
                                | Node::Transition { ref mut children, .. }
                                | Node::BeginningOfLine { ref mut children }
                                | Node::EndOfLine { ref mut children }
                                | Node::MatchOne { ref mut children, .. }
                                | Node::MatchAll { ref mut children } => {
                                    children.push(after_index.clone());
                                }
                                _ => panic!("something went wrong"),
                            }
                            map.insert(current_node_index.clone(), current_node);
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
                            callstack.pop();
                            let upcoming = upcoming_transition_stack.pop().unwrap();
                            callstack.push(upcoming);
                            // println!("After ) Operator {:?}", callstack);
                        }
                        '[' => {
                            collecting_range = true;
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
                                | Node::MatchAll { ref mut children } => {
                                    children.push(after_index.clone());
                                }
                                _ => panic!("something went wrong"),
                            }
                            map.insert(current_node_index.clone(), current_node);
                            callstack.pop();
                            // println!("After | Operator {:?}", callstack);
                        }
                        '+' => {
                            let lookback = string[string_index - 1];
                            if lookback == ')' || lookback == ']' {
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
                                    | Node::MatchAll { ref mut children } => {
                                        children.push(x.clone());
                                    }
                                    _ => panic!("Addition Compile error"),
                                }
                                map.insert(x.clone(), node);
                            }
                        }
                        '*' => {
                            let lookback = string[string_index - 1];
                            if lookback == ')' || lookback == ']' {
                                let node_index = callstack.pop().unwrap();
                                let mut after = map.get(&node_index).unwrap().clone();
                                let before_index = node_index - 1;
                                match after {
                                    Node::Transition { ref mut children, .. } => {
                                        children.push(before_index);
                                    }
                                    _ => panic!("Addition Compile error"),
                                }
                                map.insert(node_index.clone(), after);
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
                                    | Node::MatchAll { ref mut children } => {
                                        children.push(node_index);
                                        map.insert(node_index + 1, node);
                                    }
                                    _ => panic!("Something went wrong here"),
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
                        '^' | '$' => add_character('\n', &mut map, &mut callstack, &mut map_index, &string, &string_index),
                        _ => {
                            add_character(character, &mut map, &mut callstack, &mut map_index, &string, &string_index);
                        }
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
                | Node::MatchOne { ref mut children, .. } => {
                    children.push(map_index);
                    map.insert(index.clone(), x);
                }
            };
            map.insert(map_index, Node::End);
            map_index += 1;
            return (map, map_index, string_index);
        }
        // println!("compiling");
        let mut map = NodeMap::new();
        map.insert(0, Node::new_transition());
        let (new_tree, _, _) = parse(
            map,
            str_to_char_vec(self.expr.as_str()),
            1,
            &mut vec![0, 0],
            &mut vec![0],
            0,
        );
        self.tree = Some(new_tree);
    }

    fn compile(&mut self) {
        let mut tree = self.tree.clone().unwrap();
        // self.tree = None;
        let root = CompiledNode::map_to_compiled_node_tree(&mut tree);
        self.compiled_root = Some(root);
    }
}
