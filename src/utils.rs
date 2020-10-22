pub fn str_to_char_vec(string: &str) -> Vec<char> {
    let mut vec = Vec::with_capacity(string.len());
    for c in string.chars() {
        vec.push(c);
    }
    return vec;
}

pub fn calculate_range() {}
