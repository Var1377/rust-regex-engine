use super::node::*;
use super::regex::*;
use super::utils::*;
use num_cpus;
use std::sync::{mpsc::channel, Arc};
use threadpool::ThreadPool;

impl Regex {
    pub fn match_str(&self, string: &str) -> bool {
        fn _match(map: &NodeMap, chars: &Vec<char>, node_index: usize, char_index: usize) -> bool {
            let node = map.get(&node_index).unwrap();
            match node {
                Node::Transition { children, .. } => {
                    // println!("Transition Node");
                    for child in children {
                        if _match(map, chars, child.clone(), char_index) {
                            return true;
                        }
                    }
                    return false;
                }
                Node::Inclusive { characters, children, .. } => {
                    if char_index > 0 {
                        if char_index == chars.len() {
                            return false;
                        }
                    }
                    let to_match = chars[char_index];
                    // println!(
                    //     "Inclusive match, regex: {:?}, comparison: {}, result {}",
                    //     characters,
                    //     to_match,
                    //     characters.contains(&to_match)
                    // );
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
                    // println!("Hit the end. It's a match");
                    return true;
                }
                Node::Exclusive { characters, children, .. } => {
                    if char_index > 0 {
                        if char_index == chars.len() {
                            // println!("Ran out of chars. Not a match");
                            return false;
                        }
                    }
                    let to_match = chars[char_index];
                    // println!(
                    //     "Inclusive match, regex: {:?}, comparison: {}, result {}",
                    //     characters,
                    //     to_match,
                    //     !characters.contains(&to_match)
                    // );
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
                    for child in children {
                        if _match(map, chars, child.clone(), char_index + 1) {
                            return true;
                        }
                    }
                    return false;
                }
            }
            // println!(
            //     "regex: {:?}, comparison: {}, result: {}",
            //     node.characters,
            //     to_match,
            //     node.characters.contains(&to_match)
            // );
        }
        let mut chars = str_to_char_vec(string);
        if self.multithreading {
            let cpus = num_cpus::get() - 1;
            let pool = ThreadPool::new(cpus);
            let num_jobs = string.len();

            let (tx, rx) = channel();
            let arc_tree = Arc::new(self.tree.clone());
            let chars = Arc::new(chars);
            for i in 0..num_jobs {
                println!("Checking {}", i);
                let tx2 = tx.clone();
                let tree = Arc::clone(&arc_tree);
                let chars = Arc::clone(&chars);
                pool.execute(move || {
                    if _match(&tree, &chars, 0, i) {
                        tx2.send(true).expect("Threading error")
                    } else {
                        tx2.send(false).expect("Threading error");
                    }
                })
            }
            loop {
                match rx.recv() {
                    Ok(c) => match c {
                        true => return true,
                        false => continue,
                    },
                    Err(_) => {
                        break;
                    }
                }
            }
            return false;
        } else {
            for i in 0..chars.len() {
                if _match(&self.tree, &chars, 0, i) {
                    return true;
                }
            }
            return false;
        }
    }

    pub fn match_string(&self, string: String) -> bool {
        return self.match_str(string.as_str());
    }
}
