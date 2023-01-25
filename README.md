# hx

Convert binary to ASCII hexadecimal strings.

This was a coding exercise for me to learn `cargo bench` and how to
write [inline assembly][rust-asm] in Rust.

Originally, it was for me to learn how to use Rust's portable SIMD
module, but I ended up writing inline assembly instead ¯\\_(ツ)\_/¯

> Writing Rust portable SIMD sure is easy! [Same energy](https://twitter.com/0xricksanchez/status/1554895420035403776)

[rust-asm]: https://doc.rust-lang.org/nightly/rust-by-example/unsafe/asm.html

# Performance

Here are the benchmarks on my M1 Macbook (note: I am not diligent about
keeping a quiet machine). Each test was run on 12 MiB of input:

| Implementation      | Average time per iteration | Stddev      | Throughput  |
|---------------------|---------------------------:|------------:|------------:|
| [byte\_by\_byte][a] |              14,855,262 ns | ±306,213 ns | 0.79 GiB/s  |
| [portable\_simd][b] |               3,385,895 ns | ±122,011 ns | 3.46 GiB/s  |
| [neon\_tbl][c]      |               1,876,648 ns |  ±81,677 ns | 6.24 GiB/s  |

The absolute best performance I got was **1,794,971 nanoseconds** to convert
**12 MiB of data** to ASCII hexadecimal, or a rate of **6.53 GiB/second**.
I don't have precise enough equipment to measure bytes converted per clock
cycle. I am, after all, benchmarking on my laptop, on battery power, so take
these results with a grain of salt.

[a]: ./src/implementations/byte_by_byte.rs
[b]: ./src/implementations/portable_simd.rs
[c]: ./src/implementations/neon_tbl.rs

# License

Copyright © 2023 Eddie Antonio Santos. GPLv3 licensed.
