//! Uses [std::simd] to operate on 16 bytes at once.
//!
//! As of this writing (2023-01-24) [std::simd] still has an experimental API and requires nightly,
//! so it's very possible this code will not compile when you try to use it ¯\\_(ツ)\_/¯

use std::simd::{u8x16, SimdPartialOrd};

pub fn to_ascii_hex(input: &[u8]) -> String {
    let mut buffer = Vec::with_capacity(input.len() * 2);
    let (low, middle, high) = input.as_simd();

    let low = super::byte_by_byte::_byte_by_byte(low);
    buffer.extend(low);

    for window in middle {
        let (result_one, result_two) = to_hex_simd_1(*window);
        let bytes_one = result_one.to_array();
        let bytes_two = result_two.to_array();
        buffer.extend(&bytes_one);
        buffer.extend(&bytes_two);
    }

    let high = super::byte_by_byte::_byte_by_byte(high);
    buffer.extend(&high);

    debug_assert_eq!(buffer.len(), input.len() * 2);

    unsafe { String::from_utf8_unchecked(buffer) }
}

/// Given one vector of 16 bytes, converts the text to ASCII hexadecimal bytes.
/// Converting bytes to hexadecimal will double its length, so the first vector returned is for the
/// first 8 bytes of input, and the second vector is for the latter 8 bytes of input.
#[inline]
fn to_hex_simd_1(window: u8x16) -> (u8x16, u8x16) {
    let nibble_size = u8x16::splat(4);
    let low_nibble_mask = u8x16::splat(0xF);

    // Take a vector [█,█,█,█]
    // and convert it to a vector of upper nibbles [▀,▀,▀,▀] and lower nibbles [▄,▄,▄,▄].
    let high_nibbles = window >> nibble_size;
    let low_nibbles = window & low_nibble_mask;

    // Convert the upper and lower nibbles to ASCII separately:
    // to_hex_digit([▀,▀,▀,▀]) -> [C,F,B,B]
    // to_hex_digit([▄,▄,▄,▄]) -> [A,E,A,E]
    let high_nibbles = to_hex_digit(high_nibbles);
    let low_nibbles = to_hex_digit(low_nibbles);

    // Put the vectors back into the right order:
    // [C,F,B,B], [A,E,A,E] -> [C,A,F,E], [B,A,B,E]
    high_nibbles.interleave(low_nibbles)
}

/// Takes a vector of nibbles (4 bits) and arithmetically converts them to an ASCII representation.
#[inline]
fn to_hex_digit(nibbles: u8x16) -> u8x16 {
    // We'll add these to the nibble to get the ASCII character we want.
    // We have to subtract 10 from 'A' because all nibbles that use alphabetic characters are
    // already 10 or greater.
    let letter_offset = u8x16::splat(b'A' - 10);
    let digit_offset = u8x16::splat(b'0');
    let ten = u8x16::splat(10);

    // Is nibble > 10?
    let needs_a_letter = nibbles.simd_ge(ten);
    // If so, use a letter, otherwise use digits:
    let base = needs_a_letter.select(letter_offset, digit_offset);

    base + nibbles
}
