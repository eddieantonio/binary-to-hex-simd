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

| Implementation      | Average time per iteration | Stddev      |
|---------------------|---------------------------:|------------:|
| [byte\_by\_byte][a] |              14,855,262 ns | ±306,213 ns |
| [portable\_simd][b] |               3,385,895 ns | ±122,011 ns |
| [neon\_tbl][c]      |               1,876,648 ns |  ±81,677 ns |

[a]: ./src/implementations/byte_by_byte.rs
[b]: ./src/implementations/portable_simd.rs
[c]: ./src/implementations/neon_tbl.rs

# License

Copyright © 2023 Eddie Antonio Santos. GPLv3 licensed.
