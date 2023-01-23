use std::fs;

fn main() {
    let filename = std::env::args().nth(1).unwrap();
    let file = fs::read(filename).unwrap();
    let buffer = print_hex::to_hex_using_tbl32(&file);
    println!("{buffer}");
}
