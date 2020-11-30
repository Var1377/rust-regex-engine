use super::nfa::*;
use super::fxhash::FxHashMap;

const OPTIMIZATION_PASSES: u8 = 3;

pub(crate) fn optimize(nodes: &mut Vec<Node>) {
    // Removing most transition nodes => removing vast majority of epsilon transitons. Makes it much faster.
    // Normally a 2-3x speed up
    let mut transition_state_indices = Vec::new();
    let mut transition_children = FxHashMap::default();
    for _ in 0..OPTIMIZATION_PASSES {
        for i in 0..nodes.len() {
            let node = nodes.get_mut(i).unwrap();
            match node {
                Node::Transition { ref children } => {
                    transition_state_indices.push(i);
                    let mut new_children = children.clone();
                    new_children.reverse();
                    transition_children.insert(i, new_children);
                }
                _ => (),
            }
        }
        for node in &mut *nodes {
            match node.get_children_mut() {
                Some(children) => {
                    for i in 0..children.len() {
                        let j = children[i];
                        if transition_state_indices.contains(&j) {
                            let new_children = transition_children.get(&j).unwrap();
                            children.remove(i);
                            for child in new_children {
                                if !children.contains(child) {
                                    children.insert(i, *child);
                                }
                            }
                        }
                    }
                }
                None => (),
            }
        }
    }
}
