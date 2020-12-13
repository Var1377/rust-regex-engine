use super::nfa::*;

struct Capture {
    string: Vec<char>,
    children: Vec<Capture>,
}

#[inline]
fn iter_child(callstack: &mut Vec<(usize, usize, usize, bool)>, children: &[usize], child: usize, str_index: usize) {
    match children.get(child) {
        Some(n) => {
            let (_, child, _, just_inserted) = callstack.last_mut().unwrap();
            *child += 1;
            *just_inserted = false;
            callstack.push((*n, 0, str_index, true));
        }
        None => {
            callstack.pop();
        }
    }
}

pub(crate) fn pure_match(node_vec: &[Node], chars: &[char], start_index: usize, mut callstack: &mut Vec<(usize, usize, usize, bool)>) -> bool {
    // Callstack: Vec<(node_index, child, char_index, just_inserted)>
    let mut recurstion_stack: Vec<usize> = Vec::new();
    let mut completed_recurstion_stack: Vec<usize> = Vec::new();
    let mut lookahead_stack = Vec::new();
    callstack.push((0usize, 0usize, start_index, true));
    loop {
        match callstack.last() {
            None => {
                callstack.clear();
                return false;
            },
            Some(x) => {
                // println!("New Node");
                let (node_index, mut child, mut string_index, just_inserted) = x.clone();
                let node = node_vec.get(node_index).unwrap();
                match node {
                    Node::DropStack { ref children } => {
                        if just_inserted {
                            callstack.clear();
                            callstack.push((node_index, child, string_index, false));
                        }
                        iter_child(&mut callstack, children, child, string_index);
                    }
                    Node::MatchOne { ref character, ref children } => {
                        if just_inserted {
                            match chars.get(string_index) {
                                Some(c) => {
                                    if c == character {
                                        iter_child(&mut callstack, children, child, string_index + 1);
                                    } else {
                                        callstack.pop();
                                    }
                                }
                                None => {
                                    callstack.pop();
                                }
                            }
                        } else {
                            iter_child(&mut callstack, children, child, string_index + 1);
                        }
                    }
                    Node::NotMatchOne { ref character, ref children } => {
                        if just_inserted {
                            let c = chars.get(string_index);
                            if let Some(c) = c {
                                if c != character {
                                    iter_child(&mut callstack, children, child, string_index + 1);
                                } else {
                                    callstack.pop();
                                }
                            } else {
                                callstack.pop();
                            }
                        } else {
                            iter_child(&mut callstack, children, child, string_index + 1);
                        }
                    }
                    Node::Inclusive {
                        ref children,
                        ref characters,
                    } => {
                        // println!("match one: {}, visited: {}", character, !just_inserted);
                        if just_inserted {
                            let c = chars.get(string_index);
                            if let Some(c) = c {
                                // println!("match one, not visited, valid string index: {}, {}", character, c);
                                if characters.contains(c) {
                                    iter_child(&mut callstack, children, child, string_index + 1);
                                } else {
                                    callstack.pop();
                                }
                            } else {
                                callstack.pop();
                            }
                        } else {
                            // println!("match one, visited");
                            iter_child(&mut callstack, children, child, string_index + 1);
                        }
                    }
                    Node::Exclusive {
                        ref children,
                        ref characters,
                    } => {
                        // println!("match one: {}, visited: {}", character, !just_inserted);
                        if just_inserted {
                            let c = chars.get(string_index);
                            if let Some(c) = c {
                                // println!("match one, not visited, valid string index: {}, {}", character, c);
                                if !characters.contains(c) {
                                    iter_child(&mut callstack, children, child, string_index + 1);
                                } else {
                                    callstack.pop();
                                }
                            } else {
                                callstack.pop();
                            }
                        } else {
                            // println!("match one, visited");
                            iter_child(&mut callstack, children, child, string_index + 1);
                        }
                    }
                    Node::MatchAll { ref children } => {
                        iter_child(&mut callstack, children, child, string_index + 1);
                    }
                    Node::Transition { ref children } | Node::CapGroup { ref children, .. } => {
                        if just_inserted {}
                        iter_child(&mut callstack, children, child, string_index);
                    }

                    Node::BeginningOfLine { ref children } => {
                        if just_inserted {
                            if let Some(c) = chars.get(string_index) {
                                if string_index == 0 {
                                    iter_child(&mut callstack, children, child, string_index);
                                } else if chars[string_index - 1] == '\n' {
                                    iter_child(&mut callstack, children, child, string_index);
                                } else {
                                    callstack.pop();
                                }
                            } else {
                                callstack.pop();
                            }
                        } else {
                            iter_child(&mut callstack, children, child, string_index);
                        }
                    }
                    Node::EndOfLine { ref children } => {
                        if just_inserted {
                            if string_index == chars.len() {
                                iter_child(&mut callstack, children, child, string_index);
                            } else if chars[string_index + 1] == '\n' {
                                iter_child(&mut callstack, children, child, string_index);
                            } else {
                                callstack.pop();
                            }
                        } else {
                            iter_child(&mut callstack, children, child, string_index);
                        }
                    }
                    Node::GlobalRecursion => {
                        println!("Recursive Node");
                        callstack.pop();
                        recurstion_stack.push(node_index - 1);
                        callstack.push((0,0,string_index,true));
                    }
                    Node::End => {
                        println!("End Node");
                        // println!("Index found: {}", string_index);
                        if just_inserted {
                            match recurstion_stack.pop() {
                                Some(x) => {
                                    completed_recurstion_stack.push(x);
                                    callstack.push((x,0,string_index,false));
                                }
                                None => {
                                    callstack.clear();
                                    return true;
                                }
                            }
                        } else {
                            callstack.pop();
                            match completed_recurstion_stack.pop() {
                                Some(x) => {
                                    recurstion_stack.push(x);
                                    callstack.pop();
                                }
                                None => panic!()
                            }
                        }
                    }
                    Node::StartLookAhead {ref children} => {
                        if just_inserted {
                            lookahead_stack.push(string_index);
                        }
                        iter_child(callstack, children, child, string_index);
                    }
                    Node::EndLookAhead {ref children} => {
                        let idx = lookahead_stack.pop().unwrap();
                        iter_child(callstack, children, child, idx);
                    }
                    Node::StartNegativeLookAhead {
                        ref children
                    } => {
                        if just_inserted {
                            iter_child(callstack, children, child, string_index);
                        } else {
                            match children.get(child) {
                                Some(n) => {
                                    let (_, child, _, just_inserted) = callstack.last_mut().unwrap();
                                    *child += 1;
                                    *just_inserted = false;
                                    callstack.push((*n, 0, string_index, true));
                                }
                                None => {
                                    callstack.pop();
                                    callstack.push((node_index + 1,0,string_index,false));
                                }
                            }
                        }
                    }
                    Node::EndNegativeLookAhead { ref children } => {
                        if just_inserted {
                            loop {
                                match callstack.pop() {
                                    Some(x) => {
                                        let (before_index,_,_,_) = x;
                                        if before_index == node_index - 1 {
                                            break;
                                        }
                                    },
                                    None => {
                                        callstack.clear();
                                        return false
                                    }
                                }
                            }
                        } else {
                            iter_child(callstack, children, child, string_index);
                        }
                    }
                    Node::ExclusiveUnicodeRange {ref start, ref end, ref children} => {
                        if just_inserted {
                            match chars.get(string_index) {
                                Some(c) => {
                                    let c = *c as u32;
                                    if c <= *start || c >= *end {
                                        iter_child(callstack, children , child, string_index);
                                    } else {
                                        callstack.pop();
                                    }
                                }
                                None => {
                                    callstack.pop();
                                }
                            }
                        } else {
                            iter_child(callstack, children, child, string_index);
                        }
                    }
                    Node::InclusiveUnicodeRange {ref start, ref end, ref children} => {
                        if just_inserted {
                            match chars.get(string_index) {
                                Some(c) => {
                                    let c = *c as u32;
                                    if c >= *start && c <= *end {
                                        iter_child(callstack, children , child, string_index);
                                    } else {
                                        callstack.pop();
                                    }
                                }
                                None => {
                                    callstack.pop();
                                }
                            }
                        } else {
                            iter_child(callstack, children, child, string_index);
                        }
                    }
                    _ => unimplemented!(),
                };
            }
        }
    }
}

// pub(crate) fn pure_match(node_vec: &[Node], chars: &[char], start_index: usize, mut callstack: &mut Vec<(usize, usize, usize, bool)>) -> bool {
//     // Callstack: Vec<(node_index, child, char_index, just_inserted)>
//     let mut recurstion_stack: Vec<usize> = Vec::new();
//     let mut completed_recurstion_stack: Vec<usize> = Vec::new();
//     let mut lookahead_stack = Vec::new();
//     callstack.push((0usize, 0usize, start_index, true));
//     loop {
//         match callstack.pop() {
//             None => {
//                 callstack.clear();
//                 return false;
//             },
//             Some(x) => {
//                 // println!("New Node");
//                 let (node_index, child, string_index, just_inserted) = x;
//                 let node = node_vec.get(node_index).unwrap();
//                 match node {
//                     Node::DropStack { ref children } => {
//                         callstack.clear();
//                         for child in children {
//                             callstack.push((*child, 0, string_index, true));
//                         }
//                     }
//                     Node::MatchOne { ref character, ref children } => {
//                         match chars.get(string_index) {
//                             Some(c) => {
//                                 if c == character {
//                                     for child in children {
//                                         callstack.push((*child, 0, string_index + 1, true));
//                                     }
//                                 }
//                             }
//                             None => ()
//                         }
//                     }
//                     Node::NotMatchOne { ref character, ref children } => {
//                         match chars.get(string_index) {
//                             Some(c) => {
//                                 if c != character {
//                                     for child in children {
//                                         callstack.push((*child, 0, string_index + 1, true));
//                                     }
//                                 }
//                             }
//                             None => ()
//                         }
//                     }
//                     Node::Inclusive {
//                         ref children,
//                         ref characters,
//                     } => {
//                         // println!("match one: {}, visited: {}", character, !just_inserted);
//                         match chars.get(string_index) {
//                             Some(c) => {
//                                 if characters.contains(c) {
//                                     for child in children {
//                                         callstack.push((*child, 0, string_index + 1, true));
//                                     }
//                                 }
//                             }
//                             None => ()
//                         }
//                     }
//                     Node::Exclusive {
//                         ref children,
//                         ref characters,
//                     } => {
//                         // println!("match one: {}, visited: {}", character, !just_inserted);
//                         match chars.get(string_index) {
//                             Some(c) => {
//                                 if !characters.contains(c) {
//                                     for child in children {
//                                         callstack.push((*child, 0, string_index + 1, true));
//                                     }
//                                 }
//                             }
//                             None => ()
//                         }
//                     }
//                     Node::MatchAll { ref children } => {
//                         match chars.get(string_index) {
//                             Some(c) => {
//                                 if *c != '\n' {
//                                     for child in children {
//                                         callstack.push((*child, 0, string_index + 1, true));
//                                     }
//                                 }
//                             }
//                             None => ()
//                         }
//                     }
//                     Node::MatchAllandNL { ref children } => {
//                         for child in children {
//                             callstack.push((*child, 0, string_index + 1, true));
//                         }
//                     }
//                     Node::Transition { ref children } | Node::CapGroup { ref children, .. } => {
//                         for child in children {
//                             callstack.push((*child, 0, string_index, true));
//                         }
//                     }
//                     Node::BeginningOfLine { ref children } => {
//                         if let Some(c) = chars.get(string_index) {
//                             if string_index == 0 {
//                                 for child in children {
//                                     callstack.push((*child, 0, string_index, true));
//                                 }
//                             } else if chars[string_index - 1] == '\n' {
//                                 for child in children {
//                                     callstack.push((*child, 0, string_index, true));
//                                 }
//                             }
//                         }
//                     }
//                     Node::EndOfLine { ref children } => {
//                         if string_index == chars.len() {
//                             for child in children {
//                                 callstack.push((*child, 0, string_index, true));
//                             }
//                         } else if string_index < chars.len() - 1 && chars[string_index + 1] == '\n' {
//                             for child in children {
//                                 callstack.push((*child, 0, string_index, true));
//                             }
//                         }
//                     }
//                     Node::GlobalRecursion => {
//                         recurstion_stack.push(node_index - 1);
//                         callstack.push((0,0,string_index,true));
//                     }
//                     Node::End => {
//                         callstack.push((node_index, child, string_index, just_inserted));
//                         if just_inserted {
//                             match recurstion_stack.pop() {
//                                 Some(x) => {
//                                     completed_recurstion_stack.push(x);
//                                     callstack.push((x,0,string_index,false));
//                                 }
//                                 None => {
//                                     callstack.clear();
//                                     return true;
//                                 }
//                             }
//                         } else {
//                             callstack.pop();
//                             match completed_recurstion_stack.pop() {
//                                 Some(x) => {
//                                     recurstion_stack.push(x);
//                                     callstack.pop();
//                                 }
//                                 None => panic!()
//                             }
//                         }
//                     }
//                     Node::StartLookAhead {ref children} => {
//                         callstack.push((node_index, child, string_index, just_inserted));
//                         if just_inserted {
//                             let (_,_,_,t) = callstack.last_mut().unwrap();
//                             *t = false;
//                             lookahead_stack.push(string_index);
//                             for child in children {
//                                 callstack.push((*child, 0, string_index, true));
//                             }
//                         } else {
//                             callstack.pop();
//                         }
//                     }
//                     Node::EndLookAhead {ref children} => {
//                         let idx = lookahead_stack.pop().unwrap();
//                         for child in children {
//                             callstack.push((*child, 0, idx, true));
//                         }
//                     }
//                     Node::StartNegativeLookAhead {
//                         ref children
//                     } => {
//                         callstack.push((node_index, child, string_index, just_inserted));
//                         if just_inserted {
//                             iter_child(callstack, children, child, string_index);
//                         } else {
//                             match children.get(child) {
//                                 Some(n) => {
//                                     let (_, child, _, just_inserted) = callstack.last_mut().unwrap();
//                                     *child += 1;
//                                     *just_inserted = false;
//                                     callstack.push((*n, 0, string_index, true));
//                                 }
//                                 None => {
//                                     callstack.pop();
//                                     callstack.push((node_index + 1,0,string_index,false));
//                                 }
//                             }
//                         }
//                     }
//                     Node::EndNegativeLookAhead { ref children } => {
//                         callstack.push((node_index, child, string_index, just_inserted));
//                         if just_inserted {
//                             loop {
//                                 match callstack.pop() {
//                                     Some(x) => {
//                                         let (before_index,_,_,_) = x;
//                                         if before_index == node_index - 1 {
//                                             break;
//                                         }
//                                     },
//                                     None => {
//                                         callstack.clear();
//                                         return false
//                                     }
//                                 }
//                             }
//                         } else {
//                             iter_child(callstack, children, child, string_index);
//                         }
//                     }
//                     Node::ExclusiveUnicodeRange {ref start, ref end, ref children} => {
//                         callstack.push((node_index, child, string_index, just_inserted));
//                         if just_inserted {
//                             match chars.get(string_index) {
//                                 Some(c) => {
//                                     let c = *c as u32;
//                                     if c <= *start || c >= *end {
//                                         iter_child(callstack, children , child, string_index);
//                                     } else {
//                                         callstack.pop();
//                                     }
//                                 }
//                                 None => {
//                                     callstack.pop();
//                                 }
//                             }
//                         } else {
//                             iter_child(callstack, children, child, string_index);
//                         }
//                     }
//                     Node::InclusiveUnicodeRange {ref start, ref end, ref children} => {
//                         callstack.push((node_index, child, string_index, just_inserted));
//                         if just_inserted {
//                             match chars.get(string_index) {
//                                 Some(c) => {
//                                     let c = *c as u32;
//                                     if c >= *start && c <= *end {
//                                         iter_child(callstack, children , child, string_index);
//                                     } else {
//                                         callstack.pop();
//                                     }
//                                 }
//                                 None => {
//                                     callstack.pop();
//                                 }
//                             }
//                         } else {
//                             iter_child(callstack, children, child, string_index);
//                         }
//                     }
//                     _ => unimplemented!(),
//                 };
//             }
//         }
//     }
// }

