#![feature(portable_simd)]
#![feature(test)]

extern crate test;

pub mod implementations;

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use test::{black_box, Bencher};

    #[test]
    fn basic_case() {
        use crate::implementations::*;
        let input = b"This is a big string literal that is over 16 bytes long\n\x00";
        let answer = "54686973206973206120626967207374\
                      72696E67206C69746572616C20746861\
                      74206973206F76657220313620627974\
                      6573206C6F6E670A00";
        assert_eq!(&answer, &byte_by_byte::to_ascii_hex(&input[..]));
        assert_eq!(&answer, &portable_simd::to_ascii_hex(&input[..]));
        assert_eq!(&answer, &neon_tbl::to_ascii_hex(&input[..]));
    }

    #[bench]
    fn benchmark_byte_by_byte(b: &mut Bencher) {
        use crate::implementations::byte_by_byte::to_ascii_hex;
        let input = fs::read("./test.bin").unwrap();
        b.iter(|| black_box(to_ascii_hex(&input)));
    }

    #[bench]
    fn benchmark_simd_1(b: &mut Bencher) {
        use crate::implementations::portable_simd::to_ascii_hex;
        let input = fs::read("./test.bin").unwrap();
        b.iter(|| black_box(to_ascii_hex(&input)));
    }

    #[bench]
    fn benchmark_simd_2(b: &mut Bencher) {
        use crate::implementations::neon_tbl::to_ascii_hex;
        let input = fs::read("./test.bin").unwrap();
        b.iter(|| black_box(to_ascii_hex(&input)));
    }
}
