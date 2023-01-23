use std::fs;

fn main() {
    let filename = std::env::args().nth(1).unwrap();
    let file = fs::read(filename).unwrap();
    let buffer = print_hex::simd_2(&file);
    println!("{buffer}");
}
