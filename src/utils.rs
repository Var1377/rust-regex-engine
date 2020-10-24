use super::{constants::*, node::Node};

pub fn str_to_char_vec(string: &str) -> Vec<char> {
    let mut vec = Vec::with_capacity(string.len());
    for c in string.bytes() {
        vec.push(c as char);
    }
    return vec;
}

pub fn parse_range(c1: char, c2: char, exclusive: bool) -> Node {
    let mut range = vec![];
    for cat in [LOWERCASE, UPPERCASE, DIGITS].iter() {
        if cat.contains(&c1) {
            let (i, j) = (cat.iter().position(|&x| x == c1).unwrap(), cat.iter().position(|&x| x == c2).unwrap());
            for x in i..=j {
                range.push(cat[x]);
            }
        }
    }
    return Node::new_from_chars(range, exclusive);
}
