use super::nfa::*;

pub(crate) fn iterative_pure_match(node_vec: &[Node], chars: &[char], start_index: usize) -> bool {
    // Callstack: Vec<(node_index, child, char_index)>
    let mut callstack = Vec::with_capacity(node_vec.len());
    callstack.push((0usize, 0usize, start_index));
    let mut lookahead_stack: Vec<usize> = vec![];
    loop {
        match callstack.last() {
            None => return false,
            Some(x) => {
                // println!("New Node");
                let (node_index, mut child, mut string_index) = x.clone();
                let node = node_vec.get(node_index).unwrap();
                let not_visited = child == 0;
                match node {
                    Node::MatchOne { ref character, ref children } => {
                        if not_visited {
                            match chars.get(string_index) {
                                Some(c) => {
                                    if c == character {
                                    match children.get(child) {
                                        Some(n) => {
                                            let (_, child, _) = callstack.last_mut().unwrap();
                                            *child += 1;
                                            callstack.push((*n, 0, string_index + 1));
                                        }
                                        None => {callstack.pop();}
                                    }
                                } else {
                                    callstack.pop();
                                }
                                }
                                None => {
                                    callstack.pop();
                                }
                            }
                            
                            // if let Some(c) = c {
                            //     if c == character {
                            //         if let Some(n) = children.get(child) {
                            //             let (_, child, _) = callstack.last_mut().unwrap();
                            //             *child += 1;
                            //             callstack.push((*n, 0, string_index + 1));
                            //         } else {
                            //             callstack.pop();
                            //         }
                            //     } else {
                            //         callstack.pop();
                            //     }
                            // } else {
                            //     callstack.pop();
                            // }
                        } else {
                            match children.get(child) {
                                None => {
                                    callstack.pop();
                                }
                                Some(n) => {
                                    let (_, child, _) = callstack.last_mut().unwrap();
                                    *child += 1;
                                    callstack.push((*n, 0, string_index + 1));
                                }
                            }
                        }
                    }
                    Node::NotMatchOne { ref character, ref children } => {
                        if not_visited {
                            let c = chars.get(string_index);
                            if let Some(c) = c {
                                if c != character {
                                    if let Some(n) = children.get(child) {
                                        let (_, child, _) = callstack.last_mut().unwrap();
                                        *child += 1;
                                        callstack.push((*n, 0, string_index + 1));
                                    } else {
                                        callstack.pop();
                                    }
                                } else {
                                    callstack.pop();
                                }
                            } else {
                                callstack.pop();
                            }
                        } else {
                            match children.get(child) {
                                None => {
                                    callstack.pop();
                                }
                                Some(n) => {
                                    let (_, child, _) = callstack.last_mut().unwrap();
                                    *child += 1;
                                    callstack.push((*n, 0, string_index + 1));
                                }
                            }
                        }
                    }
                    Node::Inclusive {
                        ref children,
                        ref characters,
                    } => {
                        // println!("match one: {}, visited: {}", character, !not_visited);
                        if not_visited {
                            let c = chars.get(string_index);
                            if let Some(c) = c {
                                // println!("match one, not visited, valid string index: {}, {}", character, c);
                                if characters.contains(c) {
                                    if let Some(n) = children.get(child) {
                                        let (_, child, _) = callstack.last_mut().unwrap();
                                        *child += 1;
                                        callstack.push((*n, 0, string_index + 1));
                                    } else {
                                        callstack.pop();
                                    }
                                } else {
                                    callstack.pop();
                                }
                            } else {
                                callstack.pop();
                            }
                        } else {
                            // println!("match one, visited");
                            match children.get(child) {
                                None => {
                                    callstack.pop();
                                }
                                Some(n) => {
                                    let (_, child, _) = callstack.last_mut().unwrap();
                                    *child += 1;
                                    callstack.push((*n, 0, string_index + 1));
                                }
                            }
                        }
                    }
                    Node::Exclusive {
                        ref children,
                        ref characters,
                    } => {
                        // println!("match one: {}, visited: {}", character, !not_visited);
                        if not_visited {
                            let c = chars.get(string_index);
                            if let Some(c) = c {
                                // println!("match one, not visited, valid string index: {}, {}", character, c);
                                if !characters.contains(c) {
                                    if let Some(n) = children.get(child) {
                                        let (_, child, _) = callstack.last_mut().unwrap();
                                        *child += 1;
                                        callstack.push((*n, 0, string_index + 1));
                                    } else {
                                        callstack.pop();
                                    }
                                } else {
                                    callstack.pop();
                                }
                            } else {
                                callstack.pop();
                            }
                        } else {
                            // println!("match one, visited");
                            match children.get(child) {
                                None => {
                                    callstack.pop();
                                }
                                Some(n) => {
                                    let (_, child, _) = callstack.last_mut().unwrap();
                                    *child += 1;
                                    callstack.push((*n, 0, string_index + 1));
                                }
                            }
                        }
                    }
                    Node::MatchAll { ref children } => match children.get(child) {
                        None => {
                            callstack.pop();
                        }
                        Some(n) => {
                            let (_, child, _) = callstack.last_mut().unwrap();
                            *child += 1;
                            callstack.push((*n, 0, string_index + 1));
                        }
                    },
                    Node::Transition { ref children, ref behaviour } => {
                        if not_visited {
                            match *behaviour {
                                BehaviourNode::None => (),
                                BehaviourNode::EndOfGroup => {}
                                BehaviourNode::LookAhead => {
                                    lookahead_stack.push(string_index);
                                }
                                BehaviourNode::LookBehind(n) => {
                                    if string_index >= n {
                                        string_index -= n;
                                    } else {
                                        callstack.pop();
                                    }
                                }
                                BehaviourNode::DropStack => {
                                    callstack = vec![(node_index, child, string_index)];
                                }
                                BehaviourNode::CaptureOn => {}
                                BehaviourNode::CaptureOff => {}
                                BehaviourNode::VariableSizeLookBehind(start, end) => {
                                    for i in start..=end {
                                        for child in children {
                                            let new_index = string_index - 1;
                                            callstack.push((*child, 0, new_index - i));
                                        }
                                    }
                                    unimplemented!();
                                }
                                BehaviourNode::EndLookAhead => {
                                    string_index = lookahead_stack.pop().unwrap();
                                }
                            }
                        }
                        if let Some(n) = children.get(child) {
                            let (_, child, _) = callstack.last_mut().unwrap();
                            *child += 1;
                            callstack.push((*n, 0, string_index));
                        } else {
                            callstack.pop();
                        }
                    }
                    Node::BeginningOfLine { ref children } => {
                        if not_visited {
                            if let Some(c) = chars.get(string_index) {
                                if *c == '\n' {
                                    if let Some(n) = children.get(child) {
                                        let (_, child, _) = callstack.last_mut().unwrap();
                                        *child += 1;
                                        callstack.push((*n, 0, string_index + 1));
                                    } else {
                                        callstack.pop();
                                    }
                                } else if string_index == 0 {
                                    if let Some(n) = children.get(child) {
                                        let (_, child, _) = callstack.last_mut().unwrap();
                                        *child += 1;
                                        callstack.push((*n, 0, string_index));
                                    } else {
                                        callstack.pop();
                                    }
                                } else {
                                    callstack.pop();
                                }
                            } else {
                                if string_index == 0 {
                                    if let Some(n) = children.get(child) {
                                        let (_, child, _) = callstack.last_mut().unwrap();
                                        *child += 1;
                                        callstack.push((*n, 0, string_index));
                                    } else {
                                        callstack.pop();
                                    }
                                } else {
                                    callstack.pop();
                                }
                            }
                        } else {
                            if let Some(n) = children.get(child) {
                                let (_, child, _) = callstack.last_mut().unwrap();
                                *child += 1;
                                callstack.push((*n, 0, string_index));
                            } else {
                                callstack.pop();
                            }
                        }
                    }
                    Node::EndOfLine { ref children } => {
                        if not_visited {
                            if let Some(c) = chars.get(string_index) {
                                if *c == '\n' {
                                    if let Some(n) = children.get(child) {
                                        let (_, child, _) = callstack.last_mut().unwrap();
                                        *child += 1;
                                        callstack.push((*n, 0, string_index + 1));
                                    } else {
                                        callstack.pop();
                                    }
                                } else if string_index == chars.len() {
                                    if let Some(n) = children.get(child) {
                                        let (_, child, _) = callstack.last_mut().unwrap();
                                        *child += 1;
                                        callstack.push((*n, 0, string_index));
                                    } else {
                                        callstack.pop();
                                    }
                                } else {
                                    callstack.pop();
                                }
                            } else {
                                if string_index == chars.len() {
                                    if let Some(n) = children.get(child) {
                                        let (_, child, _) = callstack.last_mut().unwrap();
                                        *child += 1;
                                        callstack.push((*n, 0, string_index));
                                    } else {
                                        callstack.pop();
                                    }
                                } else {
                                    callstack.pop();
                                }
                            }
                        } else {
                            if let Some(n) = children.get(child) {
                                let (_, child, _) = callstack.last_mut().unwrap();
                                *child += 1;
                                callstack.push((*n, 0, string_index));
                            } else {
                                callstack.pop();
                            }
                        }
                    }
                    Node::End => return true,
                };
            }
        }
    }
}

#[inline]
fn iterate_child(callstack: Vec<(usize, usize, usize)>) {

}

// //Returns (start_index, end_index, string_match)
// fn iterative_index_match(node_vec: &[Node], chars: &[char], start_index: usize) -> Option<(usize, usize, String)> {
//     let mut s = String::new();
//     // Callstack: Vec<(node_index, child, char_index)>
//     let mut callstack = Vec::with_capacity(node_vec.len());
//     let mut char_index = 0usize;
//     callstack.push((0usize, 0usize, start_index));
//     let mut capturing_stack = vec![false];
//     let mut lookahead_stack = vec![];
//     // To get around the borrow checker, add the new items to the callstack after dealing with the node
//     // let mut to_push = vec![];
//     loop {
//         match callstack.last() {
//             None => return None,
//             Some(x) => {
//                 println!("New Node");
//                 let (node_index, mut child, mut string_index) = x.clone();
//                 let node: &Node;
//                 node = node_vec.get(node_index).unwrap();
//                 let not_visited = child == 0;
//                 match node {
//                     Node::MatchOne { ref character, ref children } => {
//                         // println!("match one: {}, visited: {}", character, !not_visited);
//                         if not_visited {
//                             let c = chars.get(string_index);
//                             if let Some(c) = c {
//                                 // println!("match one, not visited, valid string index: {}, {}", character, c);
//                                 if c == character {
//                                     if let Some(n) = children.get(child) {
//                                         let (_, child, _) = callstack.last_mut().unwrap();
//                                         *child += 1;
//                                         callstack.push((*n, 0, string_index + 1));
//                                     } else {
//                                         callstack.pop();
//                                     }
//                                 } else {
//                                     callstack.pop();
//                                 }
//                             } else {
//                                 callstack.pop();
//                             }
//                         } else {
//                             // println!("match one, visited");
//                             match children.get(child) {
//                                 None => {
//                                     callstack.pop();
//                                 }
//                                 Some(n) => {
//                                     let (_, child, _) = callstack.last_mut().unwrap();
//                                     *child += 1;
//                                     callstack.push((*n, 0, string_index + 1));
//                                 }
//                             }
//                         }
//                     }
//                     Node::NotMatchOne { ref character, ref children } => {
//                         {
//                             // println!("not match one: {}, visited: {}", character, !not_visited);
//                             if not_visited {
//                                 let c = chars.get(string_index);
//                                 if let Some(c) = c {
//                                     // println!("not match one, not visited, valid string index: {}, {}", character, c);
//                                     if c != character {
//                                         if let Some(n) = children.get(child) {
//                                             let (_, child, _) = callstack.last_mut().unwrap();
//                                             *child += 1;
//                                             callstack.push((*n, 0, string_index + 1));
//                                         } else {
//                                             callstack.pop();
//                                         }
//                                     } else {
//                                         callstack.pop();
//                                     }
//                                 } else {
//                                     callstack.pop();
//                                 }
//                             } else {
//                                 // println!("not match one, visited");
//                                 match children.get(child) {
//                                     None => {
//                                         callstack.pop();
//                                     }
//                                     Some(n) => {
//                                         let (_, child, _) = callstack.last_mut().unwrap();
//                                         *child += 1;
//                                         callstack.push((*n, 0, string_index + 1));
//                                     }
//                                 }
//                             }
//                         }
//                     }
//                     Node::Inclusive {
//                         ref children,
//                         ref characters,
//                     } => {
//                         // println!("match one: {}, visited: {}", character, !not_visited);
//                         if not_visited {
//                             let c = chars.get(string_index);
//                             if let Some(c) = c {
//                                 // println!("match one, not visited, valid string index: {}, {}", character, c);
//                                 if characters.contains(c) {
//                                     if let Some(n) = children.get(child) {
//                                         let (_, child, _) = callstack.last_mut().unwrap();
//                                         *child += 1;
//                                         callstack.push((*n, 0, string_index + 1));
//                                     } else {
//                                         callstack.pop();
//                                     }
//                                 } else {
//                                     callstack.pop();
//                                 }
//                             } else {
//                                 callstack.pop();
//                             }
//                         } else {
//                             // println!("match one, visited");
//                             match children.get(child) {
//                                 None => {
//                                     callstack.pop();
//                                 }
//                                 Some(n) => {
//                                     let (_, child, _) = callstack.last_mut().unwrap();
//                                     *child += 1;
//                                     callstack.push((*n, 0, string_index + 1));
//                                 }
//                             }
//                         }
//                     }
//                     Node::Exclusive {
//                         ref children,
//                         ref characters,
//                     } => {
//                         // println!("match one: {}, visited: {}", character, !not_visited);
//                         if not_visited {
//                             let c = chars.get(string_index);
//                             if let Some(c) = c {
//                                 // println!("match one, not visited, valid string index: {}, {}", character, c);
//                                 if !characters.contains(c) {
//                                     if let Some(n) = children.get(child) {
//                                         let (_, child, _) = callstack.last_mut().unwrap();
//                                         *child += 1;
//                                         callstack.push((*n, 0, string_index + 1));
//                                     } else {
//                                         callstack.pop();
//                                     }
//                                 } else {
//                                     callstack.pop();
//                                 }
//                             } else {
//                                 callstack.pop();
//                             }
//                         } else {
//                             // println!("match one, visited");
//                             match children.get(child) {
//                                 None => {
//                                     callstack.pop();
//                                 }
//                                 Some(n) => {
//                                     let (_, child, _) = callstack.last_mut().unwrap();
//                                     *child += 1;
//                                     callstack.push((*n, 0, string_index + 1));
//                                 }
//                             }
//                         }
//                     }
//                     Node::MatchAll { ref children } => match children.get(child) {
//                         None => {
//                             callstack.pop();
//                         }
//                         Some(n) => {
//                             let (_, child, _) = callstack.last_mut().unwrap();
//                             *child += 1;
//                             callstack.push((*n, 0, string_index + 1));
//                         }
//                     },
//                     Node::Transition { ref children, ref behaviour } => {
//                         if not_visited {
//                             match *behaviour {
//                                 BehaviourNode::None => (),
//                                 BehaviourNode::EndOfGroup => {
//                                     capturing_stack.pop();
//                                 }
//                                 BehaviourNode::LookAhead => {
//                                     lookahead_stack.push(string_index);
//                                     capturing_stack.push(false);
//                                 }
//                                 BehaviourNode::LookBehind(n) => {
//                                     capturing_stack.push(false);
//                                     if string_index >= n {
//                                         string_index -= n;
//                                     } else {
//                                         callstack.pop();
//                                     }
//                                 }
//                                 BehaviourNode::DropStack => {
//                                     callstack = vec![(node_index, child, string_index)];
//                                 }
//                                 BehaviourNode::CaptureOn => {
//                                     capturing_stack.push(true);
//                                 }
//                                 BehaviourNode::CaptureOff => {
//                                     capturing_stack.push(false);
//                                 }
//                                 BehaviourNode::VariableSizeLookBehind(start, end) => {
//                                     // capturing_stack.push(false);
//                                     // for i in start..=end {
//                                     //     for child in children {
//                                     //         let new_index = string_index - 1;
//                                     //         callstack.push((*child, 0, new_index - i));
//                                     //     }
//                                     // }
//                                     unimplemented!();
//                                 }
//                                 BehaviourNode::EndLookAhead => {
//                                     string_index = lookahead_stack.pop().unwrap();
//                                 }
//                             }
//                         }
//                         if let Some(n) = children.get(child) {
//                             let (_, child, _) = callstack.last_mut().unwrap();
//                             *child += 1;
//                             callstack.push((*n, 0, string_index));
//                         } else {
//                             callstack.pop();
//                         }
//                     }
//                     Node::BeginningOfLine { ref children } => {
//                         if let Some(c) = chars.get(string_index) {
//                             if *c == '\n' {
//                                 if let Some(n) = children.get(child) {
//                                     let (_, child, _) = callstack.last_mut().unwrap();
//                                     *child += 1;
//                                     callstack.push((*n, 0, string_index + 1));
//                                 } else {
//                                     callstack.pop();
//                                 }
//                             } else if string_index == 0 {
//                                 if let Some(n) = children.get(child) {
//                                     let (_, child, _) = callstack.last_mut().unwrap();
//                                     *child += 1;
//                                     callstack.push((*n, 0, string_index));
//                                 } else {
//                                     callstack.pop();
//                                 }
//                             } else {
//                                 callstack.pop();
//                             }
//                         } else {
//                             if string_index == 0 {
//                                 if let Some(n) = children.get(child) {
//                                     let (_, child, _) = callstack.last_mut().unwrap();
//                                     *child += 1;
//                                     callstack.push((*n, 0, string_index + 1));
//                                 } else {
//                                     callstack.pop();
//                                 }
//                             } else {
//                                 callstack.pop();
//                             }
//                         }
//                     }
//                     Node::EndOfLine { ref children } => {
//                         if let Some(c) = chars.get(string_index) {
//                             if *c == '\n' {
//                                 if let Some(n) = children.get(child) {
//                                     let (_, child, _) = callstack.last_mut().unwrap();
//                                     *child += 1;
//                                     callstack.push((*n, 0, string_index + 1));
//                                 } else {
//                                     callstack.pop();
//                                 }
//                             } else if string_index == chars.len() {
//                                 if let Some(n) = children.get(child) {
//                                     let (_, child, _) = callstack.last_mut().unwrap();
//                                     *child += 1;
//                                     callstack.push((*n, 0, string_index));
//                                 } else {
//                                     callstack.pop();
//                                 }
//                             } else {
//                                 callstack.pop();
//                             }
//                         } else {
//                             if string_index == chars.len() {
//                                 if let Some(n) = children.get(child) {
//                                     let (_, child, _) = callstack.last_mut().unwrap();
//                                     *child += 1;
//                                     callstack.push((*n, 0, string_index + 1));
//                                 } else {
//                                     callstack.pop();
//                                 }
//                             } else {
//                                 callstack.pop();
//                             }
//                         }
//                     }
//                     Node::End => return Some((start_index, string_index, s)),
//                 };
//             }
//         }
//     }
// }
