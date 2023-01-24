#![feature(portable_simd)]
#![feature(test)]

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

extern crate test;

pub mod implementations;

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

        // Generate the shared test data on demand.
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

        // Generic implementation to benchmark an implementation.
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
