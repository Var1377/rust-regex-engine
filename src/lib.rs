#![allow(dead_code, unused_mut, soft_unstable)]
#![feature(test)]

extern crate num_cpus;
extern crate test;

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    use test::Bencher;
    #[test]
    fn basic_test() {
        let r = Regex::new("hello".to_string());
        assert_eq!(r.match_str("hello"), true);
        assert_eq!(r.match_str("hi"), false);
        assert_eq!(r.match_str("hell"), false);
    }

    #[test]
    fn add_operator() {
        let r = Regex::new("a+b".to_string());
        println!("{:?}", r.tree);
        assert_eq!(r.match_str("ab"), true);
        assert_eq!(r.match_str("aaaaaaaaaaaaaaaaaaaaaaab"), true);
        assert_eq!(r.match_str("no"), false);
    }

    #[test]
    fn or_operator() {
        let r = Regex::new("(a|b|c)".to_string());
        println!("{:?}", r.tree);
        assert_eq!(r.match_str("ab"), true);
        assert_eq!(r.match_str("a"), true);
        assert_eq!(r.match_str("b"), true);
        assert_eq!(r.match_str("f"), false);
    }

    #[test]
    fn in_the_middle() {
        let r = Regex::new("abc".to_string());
        println!("{:?}", r.tree);
        assert_eq!(r.match_str("ksjfdweriwukjdkabcdkjaifejs"), true);
        assert_eq!(r.match_str("ksjfdweriwukjdkadkbjaifcejs"), false);
    }

    #[test]
    fn star_operator() {
        let r = Regex::new("abcd*e".to_string());
        println!("{:?}", r.tree);
        assert_eq!(r.match_str("abcddddddddddddddddddddddddddde"), true);
        assert_eq!(r.match_str("abcddddddddddddddddddddddddddd"), false);
        assert_eq!(r.match_str("abce"), true);
    }

    #[test]
    fn add_and_star_with_brackets() {
        let r = Regex::new("(a|b|c)*d(e|f|g)+h".to_string());
        println!("{:?}", r.tree);
        assert_eq!(r.match_str("adgh"), true);
        assert_eq!(r.match_str("aaaaaadh"), false);
        assert_eq!(r.match_str("abcabcabacbacdfh"), true);
        assert_eq!(r.match_str("deh"), true);
        assert_eq!(r.match_str("beh"), false);
    }

    #[bench]
    fn benchmark(b: &mut Bencher) {
        b.iter(|| {
            basic_test();
            add_operator();
            or_operator();
            in_the_middle();
            star_operator();
            add_and_star_with_brackets();
        });
    }
}

mod constants;
mod matcher;
mod node;
pub mod regex;
mod utils;
