use std::fs;

fn main() {
    let filename = std::env::args().nth(1).unwrap();
    let file = fs::read(filename).unwrap();
    let buffer = hx::simd_2(&file);
    println!("{buffer}");
}
