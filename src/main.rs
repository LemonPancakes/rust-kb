extern crate rust_kb;

use rust_kb::KnowledgeBase;

fn main() {
    let mut kb = KnowledgeBase::new();

    if let Ok(fact) = kb.create_fact("fact: (isa box hello);") {
        match kb.assert(fact) {
            Ok(_) => {}
            Err(_) => {}
        }
    }

    if let Ok(fact) = kb.create_fact("fact: (isa box hello);") {
        println!("{:?}", kb.ask(&fact));
    }
}
