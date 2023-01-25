//! A fast binary to hexadecimal converter.
//!
//! The implementation takes advantage of SIMD to convert binary to hexadecimal... probably faster
//! than you'll ever want it!
//!
//! # Examples
//!
//! Basic usage:
//!
//! ```
//! let input = b"\xca\xfe\xba\xbe";
//! let output = hx::to_ascii_hex(input);
//! assert_eq!(&"CAFEBABE", &output);
//! ```
#![feature(portable_simd)]
#![feature(test)]

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

extern crate test;

pub mod implementations;

/// Returns the ASCII hexadecimal representation of `bytes`.
///
/// Guarantees about the output:
///
///  * The alphabetic letters will be in UPPERCASE.
///  * The returned string will be exactly twice the length of `bytes`.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use hx::to_ascii_hex;
///
/// let bytes = b"\xf0\x9f\x92\xa9";
/// assert_eq!("F09F92A9", &to_ascii_hex(&bytes[..]));
/// ```
pub fn to_ascii_hex(bytes: &[u8]) -> String {
    if cfg!(target_feature = "neon") {
        implementations::neon_tbl::to_ascii_hex(bytes)
    } else {
        implementations::portable_simd::to_ascii_hex(bytes)
    }
}

#[cfg(test)]
mod tests {
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

    mod benchmark {
        use test::{black_box, Bencher};

        /// How much test data should be used in the benchmark:
        const TEST_DATA_SIZE: usize = 12 * MEGABYTES;
        const MEGABYTES: usize = 1024 * 1024;

        // Generate the shared test data on demand on first use.
        lazy_static! {
            /// Test data shared by all benchmarks.
            /// Just random bytes.
            static ref TEST_DATA: Vec<u8> = {
                let mut test_data = Vec::with_capacity(TEST_DATA_SIZE);
                // Create test data 16 bytes at a time. Gotta go fast!
                let bytes_per_u128 = 128 / 8;
                let u128s_needed = TEST_DATA_SIZE / bytes_per_u128;

                for _ in 0..u128s_needed {
                    let random_bytes = fastrand::u128(..).to_ne_bytes();
                    test_data.extend(random_bytes);
                }
                assert_eq!(TEST_DATA_SIZE, test_data.len());

                test_data
            };
        }

        // Benchmark the given implementation name with the test data.
        macro_rules! benchmark {
            ($name: ident) => {
                #[bench]
                fn $name(b: &mut Bencher) {
                    use crate::implementations::$name::to_ascii_hex;
                    let data = &TEST_DATA;
                    b.iter(|| black_box(to_ascii_hex(data)));
                }
            };
        }

        benchmark!(byte_by_byte);
        benchmark!(portable_simd);
        benchmark!(neon_tbl);
    }
}
