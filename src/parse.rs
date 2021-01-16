use super::{constants::*, nfa::*, optimize::*, regex::*, utils::*, *, compiled_node::CompiledNode};

enum ParseMode {
    SquareBrackets(Vec<char>, u16),
    CurlyBrackets(Vec<char>),
    Escaped,
    Normal,
    Comment,
}

impl Regex {
    pub(crate) fn parse_expression(&mut self) {
        let mut nodes = parse(str_to_char_vec(&self.expr));
        optimize(&mut nodes);
        let (x,y, f) = CompiledNode::compile(nodes);
        self.node_vec = x;
        self.root_node_idx = y;
        self.engine = std::sync::Mutex::new(match f {
            EngineFlag::Backtrack => Default::default(),
            EngineFlag::Other => {
               MatchingEngine::ParallelNFA {}
            }
        });
    }
}

fn parse(mut string: Vec<char>) -> Vec<Node> {
    let mut _node_vec = vec![Node::Transition {children: vec![]}, Node::End];
    let ref mut node_vec = _node_vec;
    let mut callstack = vec![0, 0];
    let mut upcoming_transition_stack = vec![1];
    let mut state_stack = vec![ParseMode::Normal];
    let mut string_index = 0;
    let mut case_insensitive = false;
    let mut comment_mode = false;
    // let mut looking_back = false;
    let mut current_cap_group = 1;
    let mut closing_bracket = false;
    while string_index < string.len() {
        // println!("{:?}", node_vec);
        let character = string[string_index];
        match state_stack.last_mut().unwrap() {
            ParseMode::Normal => {
                match character {
                    BACKSLASH => {
                        state_stack.push(ParseMode::Escaped);
                    }
                    '(' => {
                        let before_index: usize = node_vec.len();
                        let mut before = Node::new_transition();
                        let mut after = Node::new_transition();
                        let parse_rest: bool;
                        let mut remove_brackets = false;
                        if string[string_index + 1] == '?' {
                            string_index += 2;
                            match string[string_index] {
                                ':' => {
                                    parse_rest = true;
                                }
                                'R' => {
                                    let before = Node::new_transition();
                                    // let before_index = node_vec.len();
                                    let after = Node::new_transition();
                                    add_node(before, node_vec, &mut callstack);
                                    let after_index = node_vec.len();
                                    node_vec.push(after);
                                    add_node(Node::GlobalRecursion, node_vec, &mut callstack);
                                    callstack.pop();
                                    callstack.push(after_index);
                                    string_index += 2;
                                    closing_bracket = true;
                                    continue;
                                }
                                '>' => {
                                    after = Node::DropStack { children: vec![] };
                                    parse_rest = true;
                                }
                                'i' => {
                                    case_insensitive = !case_insensitive;
                                    parse_rest = false;
                                    remove_brackets = true;
                                }
                                'x' => {
                                    comment_mode = !comment_mode;
                                    parse_rest = false;
                                    remove_brackets = true;
                                }
                                '=' => {
                                    before = Node::StartLookAhead { children: vec![] };
                                    after = Node::EndLookAhead { children: vec![] };
                                    parse_rest = true;
                                }
                                '!' => {
                                    before = Node::StartNegativeLookAhead { children: vec![] };
                                    after = Node::EndNegativeLookAhead { children: vec![] };
                                    parse_rest = true;
                                }
                                _ => unimplemented!(),
                            }
                        } else {
                            before = Node::CapGroup {
                                children: Vec::new(),
                                number: current_cap_group,
                            };
                            current_cap_group += 1;
                            parse_rest = true;
                        }
                        if parse_rest {
                            node_vec.push(before);
                            let after_index: usize = node_vec.len();
                            node_vec.push(after);
                            upcoming_transition_stack.push(after_index);
                            let old = callstack.pop().unwrap();
                            let to_connect = node_vec.get_mut(old).unwrap();
                            to_connect.push_child(before_index);
                            callstack.push(before_index);
                            callstack.push(*callstack.last().unwrap());
                        } else if remove_brackets {
                            string.remove(string_index - 2);
                            string.remove(string_index - 2);
                            string.remove(string_index - 2);
                            string.remove(string_index - 2);
                        }
                    }
                    ')' => {
                        let after_index = upcoming_transition_stack.last().unwrap();
                        let current_last_node_index = callstack.pop().unwrap();
                        let current_node = node_vec.get_mut(current_last_node_index).unwrap();
                        current_node.push_child(*after_index);
                        let to_connect = upcoming_transition_stack.pop().unwrap();
                        callstack.push(to_connect);
                        closing_bracket = true;
                        string_index += 1;
                        continue;
                    }
                    '[' => {
                        state_stack.push(ParseMode::SquareBrackets(vec![], 1));
                    }
                    '|' => {
                        // println!("Before | Operator {:?}", callstack);
                        let after_index = upcoming_transition_stack.last().unwrap();
                        let current_last_node_index = callstack.last().unwrap();
                        let current_node = node_vec.get_mut(*current_last_node_index).unwrap();
                        current_node.push_child(*after_index);
                        callstack.pop();
                        callstack.push(*callstack.last().unwrap());
                        // println!("After | Operator {:?}", callstack);
                    }
                    '+' => {
                        // if previous_char_is_closing_bracket(&string_index, &string) {
                        if closing_bracket {
                            // println!("hi");
                            let last_node_index = callstack.last().unwrap();
                            let after = node_vec.get_mut(*last_node_index).unwrap();
                            let before_index = last_node_index - 1;
                            after.push_child(before_index);
                        } else {
                            // println!("hey");
                            let x = callstack.last().unwrap().clone();
                            let node = node_vec.get_mut(x).unwrap();
                            node.push_child(x);
                        }
                    }
                    '*' => {
                        // if previous_char_is_closing_bracket(&string_index, &string) {
                        if closing_bracket {
                            let last_node_index = callstack.pop().unwrap();
                            let after = node_vec.get_mut(last_node_index).unwrap();
                            let before_index = last_node_index - 1;
                            after.push_child(before_index);
                            callstack.push(before_index);
                        } else {
                            let last_node_index = callstack.pop().unwrap();
                            let mut node = node_vec.get(last_node_index).unwrap().clone();
                            let mut new_transition = Node::new_transition();
                            node.push_child(last_node_index);
                            node_vec.push(node);
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
                    '^' => add_node(Node::new_start_of_line(), node_vec, &mut callstack),
                    '$' => add_node(Node::new_end_of_line(), node_vec, &mut callstack),
                    '.' => add_node(Node::new_match_all(), node_vec, &mut callstack),
                    '?' => {
                        if ['+', '*'].contains(&string[string_index - 1]) && !check_if_escaped(&string, string_index - 1) {
                            let new_node = Node::new_transition();
                            let new_node_index = node_vec.len();
                            node_vec.push(new_node);
                            let last_index = callstack.pop().unwrap();
                            callstack.push(new_node_index);
                            node_vec.get_mut(last_index).unwrap().insert_child(new_node_index);
                        } else {
                            // if previous_char_is_closing_bracket(&string_index, &string) {
                            if closing_bracket {
                                let before_index = callstack.last().unwrap() - 1;
                                let after_index = callstack.last().unwrap().clone();
                                let before = node_vec.get_mut(before_index).unwrap();
                                before.push_child(after_index);
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
                                old_node.push_child(node_vec.len() - 1);
                                node_vec.push(old_node);
                                node_vec.push(new_transition2);
                                callstack.push(node_vec.len() - 1);
                            }
                        }
                    }
                    '{' => {
                        state_stack.push(ParseMode::CurlyBrackets(vec![]));
                        string.remove(string_index);
                        string_index -= 1;
                    }
                    _ => add_character(character, node_vec, &mut callstack),
                }
            }
            ParseMode::Escaped => {
                match character {
                    'n' => add_character('\n', node_vec, &mut callstack),
                    'd' => add_node(Node::new_from_chars(DIGITS.to_vec(), false), node_vec, &mut callstack),
                    'D' => add_node(Node::new_from_chars(DIGITS.to_vec(), true), node_vec, &mut callstack),
                    's' => add_node(Node::new_from_chars(WHITESPACE.to_vec(), false), node_vec, &mut callstack),
                    'S' => add_node(Node::new_from_chars(WHITESPACE.to_vec(), true), node_vec, &mut callstack),
                    'w' => {
                        add_node(Node::new_from_chars(W.to_vec(), false), node_vec, &mut callstack);
                    }
                    'W' => {
                        add_node(Node::new_from_chars(W.to_vec(), true), node_vec, &mut callstack);
                    }
                    'b' => {
                        add_node(Node::WordBoundary { children: vec![] }, node_vec, &mut callstack);
                    }
                    'B' => {
                        add_node(Node::NotWordBoundary { children: vec![] }, node_vec, &mut callstack);
                    }
                    'c' => {
                        string_index += 1;
                        let character = string[string_index];
                        if let Some(c) = character.to_digit(26) {
                            add_node(Node::new_from_char(c as u8 as char), node_vec, &mut callstack);
                        } else {
                            panic!("Invalid Control Character");
                        }
                    }
                    _ => {
                        add_character(character, node_vec, &mut callstack);
                    }
                };
                state_stack.pop();
            }
            ParseMode::Comment => {}
            ParseMode::CurlyBrackets(expr) => {
                if character == '}' {
                    string.remove(string_index);
                    let lazy: bool;
                    if string_index < string.len() && string[string_index] == '?' {
                        lazy = true;
                        string_index += 1;
                    } else {
                        lazy = false;
                    }
                    parse_curly_brackets(&expr, &mut string, &mut string_index, node_vec, &mut callstack, lazy);
                    string_index -= 1;
                    state_stack.pop();
                } else {
                    expr.push(string.remove(string_index));
                    string_index -= 1;
                }
            }
            ParseMode::SquareBrackets(expr, counter) => {
                if character == '[' && !check_if_escaped(&string, string_index) {
                    *counter += 1;
                }
                if character == ']' {
                    if !check_if_escaped(&string, string_index) {
                        closing_bracket = parse_square_brackets(expr, node_vec, &mut callstack);
                        state_stack.pop();
                        string_index += 1;
                        // closing_bracket = true;
                        continue;
                    } else {
                        expr.push(string[string_index - 1]);
                    }
                } else {
                    expr.push(character);
                }
            }
        }
        string_index += 1;
        closing_bracket = false;
    }
    let index = callstack.last().unwrap();
    let x = node_vec.get_mut(*index).unwrap();
    x.push_child(1);
    return _node_vec;
}
