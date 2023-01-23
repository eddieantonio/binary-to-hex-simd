//! Using portable SIMD to operate on 16 bytes at once.

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

    let high_nibbles = window >> nibble_size;
    let low_nibbles = window & low_nibble_mask;

    let high_nibbles = to_hex_digit(high_nibbles);
    let low_nibbles = to_hex_digit(low_nibbles);

    high_nibbles.interleave(low_nibbles)
}

#[inline]
fn to_hex_digit(nibbles: u8x16) -> u8x16 {
    let alpha_base = u8x16::splat(b'A' - 10);
    let digit_base = u8x16::splat(b'0');
    let ten = u8x16::splat(10);

    let use_alpha = nibbles.simd_ge(ten);
    let base = use_alpha.select(alpha_base, digit_base);

    base + nibbles
}
