#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::{env, fs};

use bench_parser::tsv;

pub fn main() {
    let path = env::args().nth(1).unwrap();
    let source_text = fs::read_to_string(&path).unwrap();
    let _ = tsv::parse(&source_text);
}
