fn main() {
    let source_file = "src/simd_tbl.c";
    cc::Build::new().file(source_file).compile("libsimdtbl.a");
    println!("cargo:rerun-if-changed={source_file}");
}
