#[derive(Clone, Debug)]
pub enum Node {
    Inclusive { children: Vec<usize>, characters: Vec<char> },
    Exclusive { children: Vec<usize>, characters: Vec<char> },
    End,
    MatchAll { children: Vec<usize> },
    Transition { children: Vec<usize> },
    BeginningOfLine { children: Vec<usize> },
    EndOfLine { children: Vec<usize> },
    MatchOne { children: Vec<usize>, character: char },
    NotMatchOne { children: Vec<usize>, character: char },
}

impl Node {
    pub fn new_transition() -> Self {
        return Node::Transition { children: vec![] };
    }

    pub fn new_match_all() -> Self {
        return Node::MatchAll { children: vec![] };
    }

    pub fn new_from_char(c: char, exclude: bool) -> Self {
        if exclude {
            return Node::NotMatchOne {
                children: vec![],
                character: c,
            };
        } else {
            return Node::MatchOne {
                children: vec![],
                character: c,
            };
        }
    }

    pub fn new_from_chars(chars: Vec<char>, exclude: bool) -> Self {
        if exclude {
            return Node::Exclusive {
                children: vec![],
                characters: chars,
            };
        } else {
            return Node::Inclusive {
                children: vec![],
                characters: chars,
            };
        }
    }

    pub fn new_end_of_line() -> Self {
        return Node::EndOfLine { children: Vec::new() };
    }

    pub fn new_start_of_line() -> Self {
        return Node::BeginningOfLine { children: Vec::new() };
    }
}
