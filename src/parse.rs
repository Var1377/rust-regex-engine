use super::{compiled_node::CompiledNode, constants::*, nfa::*, optimize::*, regex::*, utils::*, *};

enum ParseMode {
    SquareBrackets(Vec<char>, u16),
    CurlyBrackets(Vec<char>),
    Escaped,
    Normal,
    Comment,
}

#[derive(Copy, Clone, Debug,)]
pub enum ParseToken {
    S(usize),
    M {
        first: usize, before: usize, after: usize, last: usize, to_link: usize,
    },
}

impl ParseToken {
    pub fn idx(&self) -> usize {
        *match self {
            S(n) => n,
            M{to_link, ..} => to_link
        }
    }
}

use ParseToken::*;

impl Regex {
    pub(crate) fn parse_expression(&mut self) {
        let mut nodes = parse(str_to_char_vec(&self.expr));
        optimize(&mut nodes);
        let (x, y, f) = CompiledNode::compile(nodes);
        self.node_vec = x;
        self.root_node_idx = y;
        self.engine = std::sync::Mutex::new(match f {
            EngineFlag::Backtrack => Default::default(),
            EngineFlag::Other => MatchingEngine::ParallelNFA {},
        });
        self.optimized_root_node = crate::root_node_optimizer::RootNode::generate(&self.node_vec, y, None);
    }
}

fn parse(mut string: Vec<char>) -> Vec<Node> {
    let mut _node_vec = vec![Node::new_transition(), Node::End];
    let ref mut node_vec = _node_vec;
    let mut callstack = vec![S(0), S(0)];
    let mut upcoming_transition_stack = vec![1];
    let mut state_stack = vec![ParseMode::Normal];
    let mut string_index = 0;
    let mut case_insensitive = false;
    let mut comment_mode = false;
    // let mut looking_back = false;
    let mut current_cap_group = 1;
    // let mut closing_bracket = false;
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
                                    let len = node_vec.len();
                                    let v = vec![Node::Transition {children: vec![len + 1]}, Node::Transition {children: vec![len + 2]}, Node::GlobalRecursion, Node::Transition {children: vec![len + 4]}, Node::Transition {children: vec![]}];
                                    let last_index = len + 4;
                                    node_vec.extend(v);
                                    node_vec.get_mut(callstack.pop().unwrap().idx()).unwrap().push_child(len);
                                    callstack.push(M{first: len, before: len + 1, after: len + 3, last: len + 4, to_link: len + 4});
                                    string_index += 2;
                                    continue;
                                }
                                '>' => {
                                    before = Node::StartAtomic{children: vec![]};
                                    after = Node::EndAtomic { children: vec![] };
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
                            after = Node::EndCapGroup {
                                children: Vec::new(),
                                number: current_cap_group,
                            };
                            current_cap_group += 1;
                            parse_rest = true;
                        }
                        if parse_rest {
                            let len = node_vec.len();
                            after.push_child(len + 3);
                            let v = vec![Node::Transition {children: vec![len + 1]}, before, after, Node::new_transition()];
                            node_vec.get_mut(callstack.pop().unwrap().idx()).unwrap().push_child(len);
                            node_vec.extend(v);
                            callstack.push(S(len + 1));
                            callstack.push(S(len + 1));
                            upcoming_transition_stack.push(len + 2);
                        } else if remove_brackets {
                            string.remove(string_index - 2);
                            string.remove(string_index - 2);
                            string.remove(string_index - 2);
                            string.remove(string_index - 2);
                        }
                    }
                    ')' => {
                        let after_index = upcoming_transition_stack.pop().unwrap();
                        let current_last_node_index = callstack.pop().unwrap().idx();
                        node_vec.get_mut(current_last_node_index).unwrap().push_child(after_index);
                        callstack.push(M{
                            after: after_index,
                            before: after_index - 1,
                            first: after_index - 2,
                            last: after_index + 1,
                            to_link: after_index + 1,
                        });
                    }
                    '[' => {
                        state_stack.push(ParseMode::SquareBrackets(vec![], 1));
                    }
                    '|' => {
                        // println!("Before | Operator {:?}", callstack);
                        let after_index = upcoming_transition_stack.last().unwrap();
                        let current_last_node_index = callstack.pop().unwrap().idx();
                        let current_node = node_vec.get_mut(current_last_node_index).unwrap();
                        current_node.push_child(*after_index);
                        callstack.push(*callstack.last().unwrap());
                        // println!("After | Operator {:?}", callstack);
                    }
                    '+' => {
                        let possessive = string_index < string.len() - 1 && '+' == string[string_index + 1] && !check_if_escaped(&string, string_index + 1);
                        let lazy = string_index < string.len() - 1 && '?' == string[string_index + 1] && !check_if_escaped(&string, string_index + 1);
                        if lazy || possessive {
                            string_index += 1;
                        }

                        match callstack.last().unwrap() {
                            S(x) => {
                                if possessive {
                                    let mut last_node = node_vec.pop().unwrap();
                                    last_node.get_children_mut().unwrap().clear();
                                    let len = node_vec.len();
                                    last_node.push_child(len + 1);
                                    last_node.push_child(len + 2);
                                    let v = vec![Node::StartAtomic {children: vec![len + 1]}, last_node, Node::EndAtomic {children: vec![]}];
                                    node_vec.extend(v);
                                    callstack.pop();
                                    callstack.push(S(node_vec.len() - 1));
                                } else {
                                    let i = *x;
                                    if lazy {
                                        add_node(Node::new_transition(), node_vec, &mut callstack);
                                    }
                                    node_vec.get_mut(i).unwrap().push_child(i)
                                }
                            },
                            M {before, after, first, last, ..} =>  {node_vec.get_mut(*after).unwrap().lazy_dependent_insert(*before, lazy);
                                if possessive {
                                    node_vec.get_mut(*first).unwrap().to_start_atomic();
                                    node_vec.get_mut(*last).unwrap().to_end_atomic();
                                }
                            },
                        }
                    }
                    '*' => {
                        let possessive = string_index < string.len() - 1 && '+' == string[string_index + 1] && !check_if_escaped(&string, string_index + 1);
                        let lazy = string_index < string.len() - 1 && '?' == string[string_index + 1] && !check_if_escaped(&string, string_index + 1);
                        if lazy || possessive {
                            string_index += 1;
                        }

                        match callstack.last().unwrap() {
                            S(last_node_index) => {
                                if possessive {
                                    let mut last_node = node_vec.pop().unwrap();
                                    last_node.get_children_mut().unwrap().clear();
                                    let len = node_vec.len();
                                    last_node.push_child(len + 1);
                                    last_node.push_child(len + 2);
                                    let v = vec![Node::StartAtomic {children: vec![len + 1, len + 2]}, last_node, Node::EndAtomic {children: vec![]}];
                                    node_vec.extend(v);
                                    callstack.pop();
                                    callstack.push(S(node_vec.len() - 1));
                                } else {
                                    let mut node = node_vec.get(*last_node_index).unwrap().clone();
                                    let mut new_transition = Node::new_transition();
                                    node.push_child(*last_node_index);
                                    node_vec.push(node);
                                    match new_transition {
                                        Node::Transition { ref mut children, .. } => {
                                            children.push(last_node_index + 1);
                                        }
                                        _ => panic!("Something went wrong here"),
                                    }
                                    node_vec[*last_node_index] = new_transition;
                                }
                            }
                            M {first, after, last, before, ..} => {
                                node_vec.get_mut(*first).unwrap().lazy_dependent_insert(*last, lazy);
                                node_vec.get_mut(*after).unwrap().lazy_dependent_insert(*before, lazy);
                                if possessive {
                                    node_vec.get_mut(*first).unwrap().to_start_atomic();
                                    node_vec.get_mut(*last).unwrap().to_end_atomic();
                                }
                            }
                        }
                    }
                    '^' => add_node(Node::new_start_of_line(), node_vec, &mut callstack),
                    '$' => add_node(Node::new_end_of_line(), node_vec, &mut callstack),
                    '.' => add_node(Node::new_match_all(), node_vec, &mut callstack),
                    '?' => {
                        let possessive = string_index < string.len() - 1 && '+' == string[string_index + 1] && !check_if_escaped(&string, string_index + 1);
                        let lazy = string_index < string.len() - 1 && '?' == string[string_index + 1] && !check_if_escaped(&string, string_index + 1);
                        if lazy || possessive {
                            string_index += 1;
                        }

                        match callstack.last().unwrap() {
                            S(n) => {
                                let mut new_transition1 = Node::new_transition();
                                let mut new_transition2 = Node::new_transition();
                                let old = callstack.pop().unwrap().idx();
                                let mut old_node = node_vec.get(old).unwrap().clone();
                                new_transition1.push_child(node_vec.len());
                                new_transition1.lazy_dependent_insert(node_vec.len() + 1, lazy);

                                if possessive {
                                    new_transition1.to_start_atomic();
                                    new_transition2.to_end_atomic();
                                }

                                node_vec[old] = new_transition1;
                                old_node.push_child(node_vec.len() + 1);
                                node_vec.push(old_node);
                                node_vec.push(new_transition2);
                                callstack.push(S(node_vec.len() - 1));
                            },
                            M{first, last, ..} => {
                                node_vec.get_mut(*first).unwrap().push_child(*last);
                            }
                        }
                    }
                    '{' => {
                        state_stack.push(ParseMode::CurlyBrackets(vec![]));
                        string.remove(string_index);
                        continue;
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
                    let possessive: bool;
                    if string_index < string.len() && string[string_index] == '+' {
                        possessive = true;
                        lazy = false;
                    }
                    else if string_index < string.len() && string[string_index] == '?' {
                        lazy = true;
                        possessive = false;
                        string_index += 1;
                    } else {
                        lazy = false;
                        possessive = false;
                    }
                    parse_curly_brackets(&expr, node_vec, &mut callstack, lazy, possessive);
                    string_index -= 1;
                    state_stack.pop();
                } else {
                    expr.push(string.remove(string_index));
                    string_index -= 1;
                }
                string_index += 1;
                continue;
            }
            ParseMode::SquareBrackets(expr, counter) => {
                if character == '[' && !check_if_escaped(&string, string_index) {
                    *counter += 1;
                }
                if character == ']' {
                    if !check_if_escaped(&string, string_index) {
                        parse_square_brackets(expr, node_vec, &mut callstack);
                        state_stack.pop();
                        string_index += 1;
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
    }
    let index = callstack.last().unwrap();
    node_vec.get_mut(index.idx()).unwrap().push_child(1);
    // for (index, node) in node_vec.iter_mut().enumerate() {
    //     println!("{} --- {:?}", index, node);
    // }
    return _node_vec;
}