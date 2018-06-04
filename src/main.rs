extern crate nom;
extern crate weak_table;

mod kb;

use kb::knowledge_base::*;

fn main() {
    //let mut kb = KnowledgeBase::from_file("test/full.kb").unwrap();

//    let f = Fact::new(
//        kb.intern_string("isa"),
//        vec![kb.intern_string("?a"), kb.intern_string("?b")],
//        vec![],
//    );
//    let a = kb.query(&f);
//
//    for binding in a.iter() {
//        for symbol_bind in binding.iter() {
//            print!(
//                "{} : {}, ",
//                (*symbol_bind.0).to_string(),
//                (*symbol_bind.1).to_string()
//            );
//        }
//        println!("");
    let mut kb = KnowledgeBase::new();
    if let Ok(fact) = kb.create_fact("fact: (isa box hello);") {
        kb.assert(fact);

    }

    if let Ok(fact) = kb.create_fact("fact: (isa box hello);") {
        println!("{:?}",kb.ask(&fact));
    }




    //let f = Fact::new(kb.intern_string("isa"), vec![kb.intern_string("?a"), kb.intern_string("?b")]);
    //let a = kb.query(&f);

//    for binding in a.iter() {
//        for symbol_bind in binding.iter() {
//            print!("{} : {}, ",(*symbol_bind.0).to_string(),(*symbol_bind.1).to_string());
//        }
//        println!("");
//    }
}
