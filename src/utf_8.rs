// From regex crate

/// A few elementary UTF-8 encoding and decoding functions used by the matching
/// engines.
///
/// In an ideal world, the matching engines operate on `&str` and we can just
/// lean on the standard library for all our UTF-8 needs. However, to support
/// byte based regexes (that can match on arbitrary bytes which may contain
/// UTF-8), we need to be capable of searching and decoding UTF-8 on a `&[u8]`.
/// The standard library doesn't really recognize this use case, so we have
/// to build it out ourselves.
///
/// Should this be factored out into a separate crate? It seems independently
/// useful. There are other crates that already exist (e.g., `utf-8`) that have
/// overlapping use cases. Not sure what to do.
use std::char;

const TAG_CONT: u8 = 0b1000_0000;
const TAG_TWO: u8 = 0b1100_0000;
const TAG_THREE: u8 = 0b1110_0000;
const TAG_FOUR: u8 = 0b1111_0000;

/// Returns the smallest possible index of the next valid UTF-8 sequence
/// starting after `i`.
pub fn next_utf8(text: &[u8], i: usize) -> usize {
    let b = match text.get(i) {
        None => return i + 1,
        Some(&b) => b,
    };
    let inc = if b <= 0x7F {
        1
    } else if b <= 0b110_11111 {
        2
    } else if b <= 0b1110_1111 {
        3
    } else {
        4
    };
    i + inc
}

/// Decode a single UTF-8 sequence into a single Unicode codepoint from `src`.
///
/// If no valid UTF-8 sequence could be found, then `None` is returned.
/// Otherwise, the decoded codepoint and the number of bytes read is returned.
/// The number of bytes read (for a valid UTF-8 sequence) is guaranteed to be
/// 1, 2, 3 or 4.
///
/// Note that a UTF-8 sequence is invalid if it is incorrect UTF-8, encodes a
/// codepoint that is out of range (surrogate codepoints are out of range) or
/// is not the shortest possible UTF-8 sequence for that codepoint.
#[inline]
pub fn decode_utf8(src: &[u8]) -> Option<(char, usize)> {
    let b0 = match src.get(0) {
        None => return None,
        Some(&b) if b <= 0x7F => return Some((b as char, 1)),
        Some(&b) => b,
    };
    match b0 {
        0b110_00000..=0b110_11111 => {
            if src.len() < 2 {
                return None;
            }
            let b1 = src[1];
            if 0b11_000000 & b1 != TAG_CONT {
                return None;
            }
            let cp = ((b0 & !TAG_TWO) as u32) << 6 | ((b1 & !TAG_CONT) as u32);
            match cp {
                0x80..=0x7FF => char::from_u32(cp).map(|cp| (cp, 2)),
                _ => None,
            }
        }
        0b1110_0000..=0b1110_1111 => {
            if src.len() < 3 {
                return None;
            }
            let (b1, b2) = (src[1], src[2]);
            if 0b11_000000 & b1 != TAG_CONT {
                return None;
            }
            if 0b11_000000 & b2 != TAG_CONT {
                return None;
            }
            let cp = ((b0 & !TAG_THREE) as u32) << 12 | ((b1 & !TAG_CONT) as u32) << 6 | ((b2 & !TAG_CONT) as u32);
            match cp {
                // char::from_u32 will disallow surrogate codepoints.
                0x800..=0xFFFF => char::from_u32(cp).map(|cp| (cp, 3)),
                _ => None,
            }
        }
        0b11110_000..=0b11110_111 => {
            if src.len() < 4 {
                return None;
            }
            let (b1, b2, b3) = (src[1], src[2], src[3]);
            if 0b11_000000 & b1 != TAG_CONT {
                return None;
            }
            if 0b11_000000 & b2 != TAG_CONT {
                return None;
            }
            if 0b11_000000 & b3 != TAG_CONT {
                return None;
            }
            let cp = ((b0 & !TAG_FOUR) as u32) << 18 | ((b1 & !TAG_CONT) as u32) << 12 | ((b2 & !TAG_CONT) as u32) << 6 | ((b3 & !TAG_CONT) as u32);
            match cp {
                0x10000..=0x10FFFF => char::from_u32(cp).map(|cp| (cp, 4)),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Like `decode_utf8`, but decodes the last UTF-8 sequence in `src` instead
/// of the first.
pub fn decode_last_utf8(src: &[u8]) -> Option<(char, usize)> {
    if src.is_empty() {
        return None;
    }
    let mut start = src.len() - 1;
    if src[start] <= 0x7F {
        return Some((src[start] as char, 1));
    }
    while start > src.len().saturating_sub(4) {
        start -= 1;
        if is_start_byte(src[start]) {
            break;
        }
    }
    match decode_utf8(&src[start..]) {
        None => None,
        Some((_, n)) if n < src.len() - start => None,
        Some((cp, n)) => Some((cp, n)),
    }
}

fn is_start_byte(b: u8) -> bool {
    b & 0b11_000000 != 0b1_0000000
}

pub trait CharLen {
    fn len(&self) -> usize;
}

impl CharLen for char {
    fn len(&self) -> usize {
        let b = *self as u32;
        if b <= 0x7F {
            return 1;
        } else if b <= 0b110_11111 {
            return 2;
        } else if b <= 0b1110_1111 {
            return 3;
        } else {
            return 4;
        };
    }
}
