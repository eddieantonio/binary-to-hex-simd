#![feature(portable_simd)]
#![feature(test)]

extern crate test;

use std::arch::asm;
use std::simd::{u8x16, SimdPartialOrd};

mod simd_tbl {
    #[link(name = "simdtbl")]
    extern "C" {
        pub fn to_hex_using_tbl(source: *const u8, destination: *mut u8, n: usize);
        pub fn to_hex_using_tbl32(source: *const u8, destination: *mut u8, n: usize);
    }
}

pub fn to_hex_using_tbl(input: &[u8]) -> String {
    let output_size = input.len() * 2;
    let mut buffer = Vec::<u8>::with_capacity(output_size);
    let buffer_ptr = buffer.as_mut_ptr();

    unsafe {
        simd_tbl::to_hex_using_tbl(input.as_ptr(), buffer_ptr, input.len());
        buffer.set_len(output_size);
        String::from_utf8_unchecked(buffer)
    }
}

pub fn to_hex_using_tbl32(input: &[u8]) -> String {
    let output_size = input.len() * 2;
    let mut buffer = Vec::<u8>::with_capacity(output_size);
    let buffer_ptr = buffer.as_mut_ptr();

    unsafe {
        simd_tbl::to_hex_using_tbl32(input.as_ptr(), buffer_ptr, input.len());
        buffer.set_len(output_size);
        String::from_utf8_unchecked(buffer)
    }
}

pub fn byte_by_byte(input: &[u8]) -> String {
    unsafe { String::from_utf8_unchecked(_byte_by_byte(input)) }
}

pub fn _byte_by_byte(input: &[u8]) -> Vec<u8> {
    let mut vec = Vec::with_capacity(input.len() * 2);

    let to_hex_digit = |nibble| {
        if nibble >= 10 {
            b'A' + nibble - 10
        } else {
            b'0' + nibble
        }
    };

    for byte in input.iter().copied() {
        let high_nibble = byte >> 4;
        let low_nibble = byte & 0xF;

        vec.push(to_hex_digit(high_nibble));
        vec.push(to_hex_digit(low_nibble));
    }

    vec
}

pub fn simd_1(input: &[u8]) -> String {
    let mut buffer = Vec::with_capacity(input.len() * 2);
    let (low, middle, high) = input.as_simd();

    let low = _byte_by_byte(low);
    buffer.extend(low);

    for window in middle {
        let (result_one, result_two) = to_hex_simd_1(*window);
        let bytes_one = result_one.to_array();
        let bytes_two = result_two.to_array();
        buffer.extend(&bytes_one);
        buffer.extend(&bytes_two);
    }

    let high = _byte_by_byte(high);
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

pub fn simd_2(input: &[u8]) -> String {
    const LOOKUP: [u8; 16] = *b"0123456789ABCDEF";
    // How many bytes to process in one loop iteration:
    const MOUTHFUL: usize = 32;

    let output_size = input.len() * 2;
    let mut buffer = Vec::<u8>::with_capacity(output_size);

    // do the first (up to) 31 bytes byte-by-byte:
    let n_initial_bytes = input.len() % MOUTHFUL;
    let to_hex_digit = |nibble| {
        if nibble >= 10 {
            b'A' + nibble - 10
        } else {
            b'0' + nibble
        }
    };

    for byte in input[..n_initial_bytes].iter().copied() {
        let high_nibble = byte >> 4;
        let low_nibble = byte & 0xF;

        buffer.push(to_hex_digit(high_nibble));
        buffer.push(to_hex_digit(low_nibble));
    }

    let buffer_ptr = buffer[n_initial_bytes * 2..].as_mut_ptr();
    let remaining_input = &input[n_initial_bytes..];

    assert_eq!(
        0,
        remaining_input.len() % MOUTHFUL,
        "data size must be multiple of {MOUTHFUL} bytes"
    );

    // Uh oh!
    unsafe {
        asm!(
            // Load required constants:
            "ldr        q7, [{3}]",
            "movi.16b   v6, #15",

            // Offset destination by 32 bytes to facilitate these
            // instructions:
            //     stp  x,y, [reg, -#32]
            //     stp  x,y, [reg], #64
            // (register is only incremented once)
            "add       {1}, {1}, #32",

            // Load 32-bytes from the input and increment pointer:
            "2:",
            "ldp        q0, q1, [{0}], #32",

            // Split into high and low nibbles:
            "ushr.16b   v2, v0, #4",
            "and.16b    v3, v0, v6",

            // Interleave:
            "zip1.16b   v4, v2, v3",
            "zip2.16b   v5, v2, v3",

            // Lookup ASCII:
            "tbl.16b    v4, {{ v7 }}, v4",
            "tbl.16b    v5, {{ v7 }}, v5",

            // Store 32 bytes of output
            "stp        q4, q5, [{1}, #-32]",

            // Rinse and repeat for upper half
            "ushr.16b   v2, v1, #4",
            "and.16b    v3, v1, v6",
            "zip1.16b   v4, v2, v3",
            "zip2.16b   v5, v2, v3",
            "tbl.16b    v4, {{ v7 }}, v4",
            "tbl.16b    v5, {{ v7 }}, v5",
            "stp        q4, q5, [{1}], #64",

            "subs       {2}, {2}, #32",
            "b.ne       2b",

            in(reg) remaining_input.as_ptr(),
            in(reg) buffer_ptr,
            in(reg) remaining_input.len(),
            in(reg) &LOOKUP,
            clobber_abi("C"),
        );

        buffer.set_len(output_size);
        String::from_utf8_unchecked(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use test::{black_box, Bencher};

    #[test]
    fn basic_case() {
        let input = b"This is a big string literal that is over 16 bytes long\n\x00";
        let answer = "54686973206973206120626967207374\
                      72696E67206C69746572616C20746861\
                      74206973206F76657220313620627974\
                      6573206C6F6E670A00";
        assert_eq!(&answer, &byte_by_byte(&input[..]));
        assert_eq!(&answer, &simd_1(&input[..]));
        assert_eq!(&answer, &to_hex_using_tbl(&input[..]));
        assert_eq!(&answer, &to_hex_using_tbl32(&input[..]));
        assert_eq!(&answer, &simd_2(&input[..]));
    }

    #[bench]
    fn benchmark_byte_by_byte(b: &mut Bencher) {
        let input = fs::read("./test.bin").unwrap();
        b.iter(|| black_box(byte_by_byte(&input)));
    }

    #[bench]
    fn benchmark_simd_1(b: &mut Bencher) {
        let input = fs::read("./test.bin").unwrap();
        b.iter(|| black_box(simd_1(&input)));
    }

    #[bench]
    fn benchmark_simd_using_tbl(b: &mut Bencher) {
        let input = fs::read("./test.bin").unwrap();
        b.iter(|| black_box(to_hex_using_tbl(&input)));
    }

    #[bench]
    fn benchmark_simd_using_tbl32(b: &mut Bencher) {
        let input = fs::read("./test.bin").unwrap();
        b.iter(|| black_box(to_hex_using_tbl32(&input)));
    }

    #[bench]
    fn benchmark_simd_using_tbl32_rust(b: &mut Bencher) {
        let input = fs::read("./test.bin").unwrap();
        b.iter(|| black_box(simd_2(&input)));
    }
}
