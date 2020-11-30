use super::nfa::*;

struct DFA {
    nodes: Vec<Node>,
    transition_table: Vec<Vec<usize>>,
}
