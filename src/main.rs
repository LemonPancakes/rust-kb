extern crate nom;
extern crate weak_table;

mod kb;

use kb::knowledge_base::*;

fn main() {
    let kb = KnowledgeBase::from_file("test/full.kb");
    println!("{:?}", kb);
}
