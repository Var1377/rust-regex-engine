#![allow(dead_code, unused_mut, soft_unstable, unused_variables)]
#![feature(test, array_map, map_into_keys_values)]

extern crate test;

extern crate fnv;

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    use test::Bencher;

    #[test]
    fn compile_test() {
        let r = Regex::new("hi");
    }

    #[test]
    fn basic_test() {
        let r = Regex::new("hello");
        assert_eq!(r.match_str("hello"), true);
        assert_eq!(r.match_str("hi"), false);
        assert_eq!(r.match_str("hell"), false);
    }

    #[test]
    fn add_operator() {
        let r = Regex::new("a+b");
        // println!("{:?}", r.tree);
        assert_eq!(r.match_str("ab"), true);
        assert_eq!(r.match_str("aaaaaaaaaaaaaaaaaaaaaaab"), true);
        assert_eq!(r.match_str("no"), false);
    }

    #[test]
    fn or_operator() {
        // Won't work without brackets surrounding it
        let r = Regex::new("a|b|c");
        // println!("{:?}", r.tree);
        assert_eq!(r.match_str("ab"), true);
        assert_eq!(r.match_str("a"), true);
        assert_eq!(r.match_str("b"), true);
        assert_eq!(r.match_str("f"), false);
    }

    #[test]
    fn in_the_middle() {
        let r = Regex::new("abc");
        // println!("{:?}", r.tree);
        assert_eq!(r.match_str("ksjfdweriwukjdkabcdkjaifejs"), true);
        assert_eq!(r.match_str("ksjfdweriwukjdkadkbjaiabfcejs"), false);
    }

    #[test]
    fn star_operator() {
        let r = Regex::new("abcd*e");
        // println!("{:?}", r.tree);
        assert_eq!(r.match_str("abcddddddddddddddddddddddddddde"), true);
        assert_eq!(r.match_str("abcddddddddddddddddddddddddddd"), false);
        assert_eq!(r.match_str("abce"), true);
    }

    #[test]
    fn add_and_star_with_brackets() {
        let r = Regex::new("(a|b|c)*d(e|f|g)+h");
        // println!("{:?}", r.tree);
        assert_eq!(r.match_str("adgh"), true);
        assert_eq!(r.match_str("aaaaaadh"), false);
        assert_eq!(r.match_str("abcabcabacbacdfh"), true);
        assert_eq!(r.match_str("deh"), true);
        assert_eq!(r.match_str("beh"), false);
    }

    #[test]
    fn bigger_brackets() {
        let r = Regex::new(r"(hello|hi|hey) there");
        // println!("{:?}", r.tree);
        assert_eq!(r.match_str("hello there"), true);
        assert_eq!(r.match_str("hi there"), true);
        assert_eq!(r.match_str("hey there"), true);
        assert_eq!(r.match_str("h there"), false);
        assert_eq!(r.match_str("bye there"), false);
        assert_eq!(r.match_str("llo there"), false);
    }


    #[test]
    fn square_brackets() {
        let r = Regex::new("abc[def]ghi");
        println!("{:?}", r.tree);
        assert_eq!(r.match_str("abcdghi"), true);
        assert_eq!(r.match_str("abceghi"), true);
        assert_eq!(r.match_str("abcfghi"), true);
        assert_eq!(r.match_str("abcghi"), false);
        assert_eq!(r.match_str("abdeghi"), false);
    }

    #[test]
    fn range_of_chars() {
        let r = Regex::new("[a-zA-Z]");
        // println!("{:?}", r.tree);
        assert_eq!(r.match_str("g"), true);
        assert_eq!(r.match_str("G"), true);
        assert_eq!(r.match_str("9"), false);
    }
    #[test]
    fn range_of_chars_and_other() {
        let r = Regex::new("[a-zA-Z136]");
        // println!("{:?}", r.tree);
        assert_eq!(r.match_str("g"), true);
        assert_eq!(r.match_str("G"), true);
        assert_eq!(r.match_str("9"), false);
        assert_eq!(r.match_str(")"), false);
        assert_eq!(r.match_str("1"), true);
        assert_eq!(r.match_str("3"), true);
        assert_eq!(r.match_str("6"), true);
    }

    #[test]
    fn square_brackets_with_quantifiers() {
        let r = Regex::new("[a-zA-Z136]+");
        // println!("{:?}", r.tree);
        assert_eq!(r.match_str("g13az"), true);
        assert_eq!(r.match_str("G6zA1"), true);
        assert_eq!(r.match_str("9254582"), false);
        assert_eq!(r.match_str("1GES"), true);
        assert_eq!(r.match_str("3"), true);
        assert_eq!(r.match_str("6"), true);
    }

    #[test]
    fn inclusive_d() {
        let r = Regex::new(r"\d+");
        assert_eq!(r.match_str("05421345689484651326549876532163846981351"), true);
        assert_eq!(r.match_str("asdfakjsdfklasldfajsdkhljfhalsjfd"), false);
    }

    #[test]
    fn exclusive_d() {
        let r = Regex::new(r"\D+");
        assert_eq!(r.match_str("05421345689484651326549876532163846981351"), false);
        assert_eq!(r.match_str("asdfakjsdfklasldfajsdkhljfhalsjfd"), true);
    }

    #[test]
    fn inclusive_s() {
        let r = Regex::new(r"\s+");
        assert_eq!(r.match_str("        "), true);
        assert_eq!(r.match_str("a"), false);
    }

    #[test]
    fn exclusive_s() {
        let r = Regex::new(r"\S+");
        assert_eq!(r.match_str("  "), false);
        assert_eq!(r.match_str("aaadjkfalksdfujha"), true);
    }

    #[test]
    fn inclusive_w() {
        let r = Regex::new(r"\w+");
        assert_eq!(r.match_str("0a9sd87f0a8pwoeihnpva"), true);
        assert_eq!(r.match_str("                "), false);
    }

    #[test]
    fn exclusive_w() {
        let r = Regex::new(r"\W");
        assert_eq!(r.match_str("0a9sd87f0a8pwoeihnpva"), false);
        assert_eq!(r.match_str("                "), true);
    }

    #[test]
    fn question_mark() {
        let r = Regex::new("de?f");
        println!("{:?}", r.tree);
        assert_eq!(r.match_str("abcdefg"), true);
        assert_eq!(r.match_str("abcdfg"), true);
        assert_eq!(r.match_str("abcfge"), false);
        assert_eq!(r.match_str("abcfge"), false);
    }

    #[test]
    fn question_mark_with_brackets() {
        let r = Regex::new("abc(d|e|f)?hij");
        assert_eq!(r.match_str("abcdhij"), true);
        assert_eq!(r.match_str("abcehij"), true);
        assert_eq!(r.match_str("abcfhij"), true);
        assert_eq!(r.match_str("abchij"), true);
        assert_eq!(r.match_str("abchi"), false);
        assert_eq!(r.match_str("abcdefhij"), false);
        assert_eq!(r.match_str("abfhij"), false);
    }

    #[test]
    fn question_mark_with_square_brackets() {
        let r = Regex::new("abc[def]?hij");
        // println!("{:?}", r.tree);
        assert_eq!(r.match_str("abcdhij"), true);
        assert_eq!(r.match_str("abcehij"), true);
        assert_eq!(r.match_str("abcfhij"), true);
        assert_eq!(r.match_str("abchij"), true);
        assert_eq!(r.match_str("abchi"), false);
        assert_eq!(r.match_str("abcdefhij"), false);
        assert_eq!(r.match_str("abfhij"), false);
    }

    #[bench]
    fn benchmark(b: &mut Bencher) {
        // To match a phone number
        let r = Regex::new(r"^[+]*[(]?[0-9]+[)]?[-\s\.0-9]*$");
        b.iter(|| {
            assert_eq!(r.match_str("+(123)-456-78-90"), true);
            assert_eq!(r.match_str("+44-777-66-55"), true);
            assert_eq!(r.match_str("test@gmail.com"), false);
        });
    }
}

mod compiled_node;
mod constants;
mod matcher;
mod node;
mod parse;
pub mod regex;
mod utils;
