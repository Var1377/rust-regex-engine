use super::node::*;
use super::regex::*;
use super::utils::*;
#[allow(unused_imports)]
use num_cpus;
#[allow(unused_imports)]
use std::mem::drop;
#[allow(unused_imports)]
use std::sync::{mpsc::channel, Arc, Mutex};

impl Regex {
    pub fn match_str(&self, string: &str) -> bool {
        fn _match(map: &NodeMap, chars: &Vec<char>, node_index: usize, char_index: usize) -> bool {
            let node = map.get(&node_index).unwrap();
            match node {
                Node::Transition { children, .. } => {
                    for child in children {
                        if _match(map, chars, child.clone(), char_index) {
                            return true;
                        }
                    }
                    return false;
                }
                Node::Inclusive {
                    characters,
                    children,
                    ..
                } => {
                    if char_index > 0 {
                        if char_index == chars.len() {
                            return false;
                        }
                    }
                    let to_match = chars[char_index];
                    if characters.contains(&to_match) {
                        for child in children {
                            if _match(map, chars, child.clone(), char_index + 1) {
                                return true;
                            }
                        }
                        return false;
                    } else {
                        return false;
                    }
                }
                Node::End => {
                    return true;
                }
                Node::Exclusive {
                    characters,
                    children,
                    ..
                } => {
                    if char_index > 0 {
                        if char_index == chars.len() {
                            return false;
                        }
                    }
                    let to_match = chars[char_index];
                    if characters.contains(&to_match) {
                        false
                    } else {
                        for child in children {
                            if _match(map, chars, child.clone(), char_index + 1) {
                                return true;
                            }
                        }
                        return false;
                    }
                }
                Node::MatchAll { children, .. } => {
                    if char_index > 0 {
                        if char_index == chars.len() {
                            return false;
                        }
                    }
                    let to_match = chars[char_index];
                    if to_match == '\n' {
                        return false;
                    } else {
                        for child in children {
                            if _match(map, chars, child.clone(), char_index + 1) {
                                return true;
                            }
                        }
                        return false;
                    }
                }
            }
        }
        let mut chars = str_to_char_vec(string);
        // if self.multithreading {
        //     let cpus = num_cpus::get();
        //     let num_jobs = chars.len();
        //     let mut queue = vec![];

        //     for i in 0..num_jobs {
        //         queue.push(i);
        //     }

        //     let queue_arc = Arc::new(Mutex::new(queue));

        //     let (tx, rx) = channel();

        //     let arc_tree = Arc::new(self.tree.clone());

        //     let arc_chars = Arc::new(chars);

        //     let mut handles = vec![];

        //     for t in 0..cpus {
        //         println!("Starting thread: {}", t);
        //         let queue = queue_arc.clone();
        //         let tx = tx.clone();
        //         let chars = arc_chars.clone();
        //         let map = arc_tree.clone();
        //         handles.push(std::thread::spawn(move || loop {
        //             let mut q = queue.lock().unwrap();
        //             if !q.is_empty() {
        //                 let i = q.pop().unwrap();
        //                 drop(q);
        //                 let message = _match(&map, &chars, 0, i);
        //                 if message {
        //                     let mut q = queue.lock().unwrap();
        //                     q.truncate(0);
        //                     drop(q);
        //                 }
        //                 tx.send(message).unwrap();
        //             } else {
        //                 drop(q);
        //                 break;
        //             }
        //         }));
        //     }
        //     for handle in handles {
        //         handle.join().unwrap();
        //     }
        //     println!("All threads stopped");
        //     loop {
        //         let received = rx.try_recv();
        //         match received {
        //             Ok(m) => {
        //                 if m {
        //                     return true;
        //                 }
        //             }
        //             Err(_) => return false,
        //         }
        //     }
        // } else {
            for i in 0..chars.len() {
                if _match(&self.tree, &chars, 0, i) {
                    return true;
                }
            }
            return false;
        // }
    }

    pub fn match_string(&self, string: String) -> bool {
        return self.match_str(string.as_str());
    }
}
