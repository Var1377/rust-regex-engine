use super::matcher::*;
use super::regex::*;
use super::utils::*;

impl Regex {
    pub fn replace_first_str<'a>(&self, s: &'a str, r: &'a str) -> String {
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
                let (before, after) = chars.split_at(start);
                let (_, after) = after.split_at(end); 
                let mut vec = before.to_vec();
                vec.extend(replace_with);
                vec.extend(after);
                return char_vec_to_string(&vec);
            }
        }
        return s.to_string();
    }

    pub fn replace_first_string(&self, s: String, r: String) -> String {
        return self.replace_first_str(s.as_str(), r.as_str());
    }
}
