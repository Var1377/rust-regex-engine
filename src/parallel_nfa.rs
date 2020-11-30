// use super::nfa::*;
use fxhash::FxHashSet;

// fn text_driven_match(chars: &[char], nodes: &[Node], mut char_index: usize) -> bool {
//     let mut current_states: Vec<usize> = Vec::new();
//     let mut new_states: Vec<usize> = Vec::new();
//     current_states.push(0);

//     while current_states.len() > 0 {
//         let c = match chars.get(char_index) {
//             Some(c) => c,
//             None => return false,
//         };
//         let mut i = 0;
//         while i < current_states.len() {
//             let node = nodes.get(i).unwrap();
//             match node {
//                 Node::Inclusive { children, characters } => {
//                     if characters.contains(&c) {
//                         new_states.extend(children);
//                     }
//                 }
//                 Node::Exclusive { children, characters } => {
//                     if !characters.contains(&c) {
//                         new_states.extend(children);
//                     }
//                 }
//                 Node::MatchOne { children, character } => {
//                     if character == c {
//                         new_states.extend(children);
//                     }
//                 }
//                 Node::NotMatchOne { children, character } => {
//                     if character != c {
//                         new_states.extend(children);
//                     }
//                 }
//                 Node::BeginningOfLine { children } => {
//                     if *c == '\n' {
//                         new_states.extend(children);
//                     } else if char_index == 0 {
//                         current_states.extend(children);
//                     } else if chars[char_index - 1] == '\n' {
//                         current_states.extend(children);
//                     }
//                 }
//                 Node::EndOfLine { children } => {
//                     if *c == '\n' {
//                         new_states.extend(children);
//                     } else if char_index == chars.len() {
//                         current_states.extend(children);
//                     }
//                 }
//                 Node::WordBoundary { children } => {}
//                 Node::Transition { children } => {
//                     current_states.extend(children);
//                 }
//                 Node::End => return true,
//                 _ => unimplemented!(),
//             }
//             i += 1;
//         }
//         current_states.clear();
//         current_states.extend(new_states.drain(..));
//         char_index += 1;
//     }
//     return false;
// }
