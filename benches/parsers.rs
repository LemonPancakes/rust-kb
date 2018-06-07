#![feature(test)]

extern crate rust_kb;
extern crate test;

use rust_kb::kb::knowledge_base::KnowledgeBase;
use test::Bencher;

// ~6s
#[bench]
fn nom_parse(b: &mut Bencher) {
    b.iter(|| KnowledgeBase::from_file("test/large.kb"));
}
