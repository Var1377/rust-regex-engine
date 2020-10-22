use super::constants::*;
use super::node::*;
use super::utils::*;

#[derive(Clone, Debug)]
pub struct Regex {
    pub expr: String,
    pub tree: NodeMap,
    pub multithreading: bool,
}

impl Default for Regex {
    fn default() -> Self {
        let mut map = NodeMap::new();
        map.insert(0, Node::new_transition(0));
        return Regex {
            expr: String::from(""),
            tree: map,
            multithreading: false,
        };
    }
}

impl Regex {
    pub fn new(mut regex: String) -> Self {
        let mut r = Self::default();
        regex.push(')');
        let mut new_regex = "(".to_string();
        new_regex.push_str(&regex);
        regex = new_regex;
        r.expr = regex;
        r.compile();
        return r;
    }

    fn compile(&mut self) {
        fn change_transition(
            c: char,
            exclude: bool,
            map: &mut NodeMap,
            callstack: &mut Vec<usize>,
            map_index: &mut usize,
            chars: &Vec<char>,
            char_index: &usize,
        ) {
            println!("Before: {:?}", callstack);
            if char_index > &0 {
                let lookback = chars.get(char_index - 1).unwrap();
                match lookback {
                    '(' | '|' => callstack.push(callstack.last().unwrap().clone()),
                    _ => {}
                }
            }
            map.insert(map_index.clone(), Node::new_from_char(c, exclude, map_index.clone()));
            if let Some(to_connect) = callstack.pop() {
                let mut node = map.get(&to_connect).unwrap().clone();
                match node {
                    Node::Inclusive { ref mut children, .. }
                    | Node::Exclusive { ref mut children, .. }
                    | Node::Transition { ref mut children, .. } => {
                        children.push(map_index.clone());
                        map.insert(to_connect.clone(), node);
                    }
                    _ => panic!("change transition function failed"),
                }
            }
            callstack.push(map_index.clone());
            *map_index += 1;
            println!("After: {:?}", callstack);
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
            let len = string.len();
            while string_index < len {
                let character = string[string_index];
                if looking_back {
                    let lookback = string[string_index - 1];
                    match lookback {
                        BACKSLASH => match character {
                            BACKSLASH => change_transition(BACKSLASH, false, &mut map, &mut callstack, &mut map_index, &string, &string_index),
                            CARET => change_transition(CARET, false, &mut map, &mut callstack, &mut map_index, &string, &string_index),
                            DOLLAR => change_transition(DOLLAR, false, &mut map, &mut callstack, &mut map_index, &string, &string_index),
                            'n' => change_transition('\n', false, &mut map, &mut callstack, &mut map_index, &string, &string_index),
                            '+' | '*' => change_transition(character, false, &mut map, &mut callstack, &mut map_index, &string, &string_index),
                            _ => {}
                        },
                        _ => panic!("Something has gone wrong here"),
                    }
                } else {
                    match character {
                        BACKSLASH => {
                            looking_back = true;
                        }
                        '(' => {
                            let mut before = Node::new_transition(map_index);
                            map_index += 1;
                            let mut after = Node::new_transition(map_index);
                            map_index += 1;
                            let before_index: usize;
                            let after_index: usize;
                            match before {
                                Node::Transition { index, .. } => {
                                    before_index = index.clone();
                                }
                                _ => panic!("aah"),
                            };
                            match after {
                                Node::Transition { index, .. } => {
                                    after_index = index.clone();
                                }
                                _ => panic!("aah"),
                            };
                            upcoming_transition_stack.push(after_index);
                            let old = callstack.pop().unwrap();
                            let mut to_connect = map.get(&old).unwrap().clone();
                            match to_connect {
                                Node::Exclusive { ref mut children, .. }
                                | Node::Inclusive { ref mut children, .. }
                                | Node::Transition { ref mut children, .. }
                                | Node::MatchAll { ref mut children, .. } => {
                                    children.push(before_index);
                                }
                                _ => panic!("aah"),
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
                                Node::Transition { ref mut children, .. }
                                | Node::Exclusive { ref mut children, .. }
                                | Node::MatchAll { ref mut children, .. }
                                | Node::Inclusive { ref mut children, .. } => {
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
                        '[' => {}
                        '|' => {
                            // println!("Before | Operator {:?}", callstack);
                            let after_index = upcoming_transition_stack.last().unwrap();
                            let current_node_index = callstack.last().unwrap();
                            let mut current_node = map.get(current_node_index).unwrap().clone();
                            match &mut current_node {
                                Node::Transition { ref mut children, .. }
                                | Node::Exclusive { ref mut children, .. }
                                | Node::MatchAll { ref mut children, .. }
                                | Node::Inclusive { ref mut children, .. } => {
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
                            if lookback == ')' {
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
                                    Node::Inclusive { ref mut children, .. } | Node::Exclusive { ref mut children, .. } => {
                                        children.push(x.clone());
                                    }
                                    _ => panic!("Addition Compile error"),
                                }
                                map.insert(x.clone(), node);
                            }
                        }
                        '*' => {
                            let lookback = string[string_index - 1];
                            if lookback == ')' {
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
                                let mut new_transition = Node::new_transition(node_index.clone());
                                match node {
                                    Node::Inclusive {
                                        ref mut index,
                                        ref mut children,
                                        ..
                                    }
                                    | Node::Exclusive {
                                        ref mut index,
                                        ref mut children,
                                        ..
                                    }
                                    | Node::Transition {
                                        ref mut index,
                                        ref mut children,
                                        ..
                                    }
                                    | Node::MatchAll {
                                        ref mut index,
                                        ref mut children,
                                        ..
                                    } => {
                                        children.push(*index);
                                        *index += 1;
                                        map.insert(*index, node);
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
                        CARET | DOLLAR => change_transition('\n', false, &mut map, &mut callstack, &mut map_index, &string, &string_index),
                        _ => {
                            change_transition(character, false, &mut map, &mut callstack, &mut map_index, &string, &string_index);
                        }
                    }
                }
                string_index += 1;
            }
            // println!("Final {:?}", callstack);
            let mut x = map.get(callstack.last().unwrap()).unwrap().clone();
            match x {
                Node::End => {}
                Node::Exclusive { ref mut children, index, .. }
                | Node::Inclusive { ref mut children, index, .. }
                | Node::Transition { ref mut children, index, .. }
                | Node::MatchAll { ref mut children, index, .. } => {
                    children.push(map_index);
                    map.insert(index, x);
                }
            };
            map.insert(map_index, Node::End);
            map_index += 1;
            return (map, map_index, string_index);
        }
        // println!("compiling");
        let (new_tree, _, _) = parse(self.tree.clone(), str_to_char_vec(self.expr.as_str()), 1, &mut vec![0], &mut vec![0], 0);
        self.tree = new_tree;
    }
}
