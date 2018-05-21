#[macro_use]
extern crate nom;

mod kb;

use kb::parser::parse_kb_from_file;

fn main() {
    let kb = parse_kb_from_file("src/full.kb").unwrap();
    println!("{:?}", kb);
}
