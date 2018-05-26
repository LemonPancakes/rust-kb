#[macro_use]
extern crate nom;

mod kb;

use kb::knowledge_base::*;

fn main() {
    let kb = KnowledgeBase::from_file("test/full.kb");
    println!("{:?}", kb);
}
