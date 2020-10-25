use super::node::*;
use std::collections::*;
use std::rc::Rc;

// Not in use yet because I still need to solve a circular refernce issue when using * or + when converting to CompiledNodes

#[derive(Debug)]
pub enum CompiledNode {
    Inclusive {
        children: Vec<Rc<CompiledNode>>,
        characters: Vec<char>,
    },
    Exclusive {
        children: Vec<Rc<CompiledNode>>,
        characters: Vec<char>,
    },
    End,
    MatchAll {
        children: Vec<Rc<CompiledNode>>,
    },
    Transition {
        children: Vec<Rc<CompiledNode>>,
    },
    BeginningOfLine {
        children: Vec<Rc<CompiledNode>>,
    },
    EndOfLine {
        children: Vec<Rc<CompiledNode>>,
    },
    MatchOne {
        children: Vec<Rc<CompiledNode>>,
        character: char,
    },
    NotMatchOne {
        children: Vec<Rc<CompiledNode>>,
        character: char,
    },
}

impl CompiledNode {
    pub fn map_to_compiled_node_tree(map: &mut NodeMap) -> Rc<CompiledNode> {
        fn add_children(index: &usize, map: &NodeMap, cache: &mut BTreeMap<usize, Rc<CompiledNode>>) -> Rc<CompiledNode> {
            let cached = cache.get(index);
            match cached {
                Some(a) => return Rc::clone(a),
                None => (),
            };
            let node = map.get(index).unwrap();
            let mut new_node: CompiledNode;
            new_node = match node {
                Node::BeginningOfLine { .. } => CompiledNode::BeginningOfLine { children: Vec::new() },
                Node::Inclusive { .. } => CompiledNode::Inclusive {
                    children: Vec::new(),
                    characters: Vec::new(),
                },
                Node::MatchAll { .. } => CompiledNode::Exclusive {
                    children: Vec::new(),
                    characters: Vec::new(),
                },
                Node::MatchOne { character, .. } => CompiledNode::MatchOne {
                    children: Vec::new(),
                    character: *character,
                },
                Node::NotMatchOne { character, .. } => CompiledNode::NotMatchOne {
                    children: Vec::new(),
                    character: *character,
                },
                Node::Transition { .. } => CompiledNode::Transition { children: Vec::new() },
                Node::Exclusive { .. } => CompiledNode::Exclusive {
                    children: Vec::new(),
                    characters: Vec::new(),
                },
                Node::EndOfLine { .. } => CompiledNode::EndOfLine { children: Vec::new() },
                Node::End => return Rc::new(CompiledNode::End),
            };

            match node {
                Node::BeginningOfLine { ref children, .. }
                | Node::Inclusive { ref children, .. }
                | Node::Exclusive { ref children, .. }
                | Node::MatchAll { ref children, .. }
                | Node::Transition { ref children, .. }
                | Node::EndOfLine { ref children, .. }
                | Node::MatchOne { ref children, .. }
                | Node::NotMatchOne { ref children, .. } => {
                    let c = children;
                    match new_node {
                        CompiledNode::BeginningOfLine { ref mut children, .. }
                        | CompiledNode::Inclusive { ref mut children, .. }
                        | CompiledNode::Exclusive { ref mut children, .. }
                        | CompiledNode::MatchAll { ref mut children, .. }
                        | CompiledNode::Transition { ref mut children, .. }
                        | CompiledNode::EndOfLine { ref mut children, .. }
                        | CompiledNode::MatchOne { ref mut children, .. }
                        | CompiledNode::NotMatchOne { ref mut children, .. } => {
                            for child in c {
                                children.push(add_children(child, map, cache))
                            }
                        }
                        CompiledNode::End => panic!(),
                    }
                }
                Node::End => panic!(),
            }
            let ptr = Rc::new(new_node);
            let cloned = Rc::clone(&ptr);
            cache.insert(index.clone(), cloned);
            return ptr;
        }
        let root = add_children(&0, map, &mut BTreeMap::new());
        println!("{:?}", root);
        return root;
    }
}
