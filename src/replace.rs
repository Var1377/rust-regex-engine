use super::matcher::*;
use super::regex::*;
use super::utils::*;

impl Regex {
    pub fn replace_first(&self, s: &str, r: &str) -> String {
        let chars = str_to_char_vec(s);
        let mut indices: Option<(usize, usize)> = None;
        for i in 0..chars.len() {
            match get_index_match(&self.node_vec, &chars, &0, i) {
                None => (),
                Some(x) => {
                    indices = Some((i, x));
                    break;
                }
            }
        }
        match indices {
            None => (),
            Some(x) => {
                let replace_with = str_to_char_vec(r);
                let (start, end) = x;
                return char_vec_to_string(&replace_range(start, end, &replace_with, &chars));
            }
        }
        return s.to_string();
    }

    pub fn replace_first_mapped<F: FnMut(String) -> String>(&self, s: &str, mut func: F) -> String {
        let chars = str_to_char_vec(s);
        let mut indices: Option<(usize, usize)> = None;
        for i in 0..chars.len() {
            match get_index_match(&self.node_vec, &chars, &0, i) {
                None => (),
                Some(x) => {
                    indices = Some((i, x));
                    break;
                }
            }
        }
        match indices {
            None => (),
            Some(x) => {
                let (start, end) = x;
                // let mut string = String::new();
                // for i in start..=end {
                //     string.push(chars[i]);
                // }
                // let s =  func(string);
                // let replace_with = str_to_char_vec(&s);
                // let (before, after) = chars.split_at(start);
                // let (_, after) = after.split_at(end);
                // let mut vec = before.to_vec();
                // vec.extend(replace_with);
                // vec.extend(after);
                return char_vec_to_string(&replace_mapped(start, end, &mut func, &chars));
            }
        }
        return s.to_string();
    }

    pub fn replace_all(&self, s: &str, r: &str) -> String {
        let mut chars = str_to_char_vec(s);
        let mut indices: Vec<(usize, usize)> = Vec::new();
        let mut i = 0usize;
        while i < chars.len() {
            match get_index_match(&self.node_vec, &chars, &0, i) {
                None => (),
                Some(end) => {
                    indices.push((i, end));
                    i = end;
                }
            }
            i += 1;
        }
        if indices.is_empty() {
            return s.to_string();
        }
        indices.reverse();
        let replace_with = str_to_char_vec(r);
        for (start, end) in indices {
            chars = replace_range(start, end, &replace_with, &chars);
        }
        return char_vec_to_string(&chars);

    }

    pub fn replace_all_mapped<F: FnMut(String) -> String>(&self, s: &str, mut func: F) -> String {
        let mut chars = str_to_char_vec(s);
        let mut indices: Vec<(usize, usize)> = Vec::new();
        let mut i = 0usize;
        while i < chars.len() {
            match get_index_match(&self.node_vec, &chars, &0, i) {
                None => (),
                Some(end) => {
                    indices.push((i, end));
                    i = end;
                }
            }
            i += 1;
        }
        if indices.is_empty() {
            return s.to_string();
        }
        indices.reverse();
        for (start, end) in indices {
            chars = replace_mapped(start, end, &mut func, &chars);
        }
        return char_vec_to_string(&chars);
    }
}

fn replace_range(start: usize, end: usize, r: &[char], chars: &[char]) -> Vec::<char> {
    let mut out = Vec::new();
    let (v1, _) = chars.split_at(start);
    let (_, v2) = chars.split_at(end);
    out.extend(v1);
    out.extend(r);
    out.extend(v2);
    return out;
}

fn replace_mapped<F: FnMut(String) -> String>(start: usize, end: usize, func: &mut F, chars: &[char]) -> Vec::<char> {
    let mut out = Vec::new();
    let (v1, v3) = chars.split_at(start);
    let (_, v2) = chars.split_at(end);
    let (m, _) = v3.split_at(end - v1.len());
    let r = str_to_char_vec(&func(char_vec_to_string(m)));
    out.extend(v1);
    out.extend(r);
    out.extend(v2);
    return out;
}