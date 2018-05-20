extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::*;

#[cfg(debug_assertions)]
const _GRAMMAR: &'static str = include_str!("kb.pest");

#[derive(Parser)]
#[grammar = "kb.pest"]
pub struct PDDLParser;



fn main() {
    let x = PDDLParser::parse(Rule::kb,"fact: (isa cube block)\nfact: (isa box container)\nrule: ((inst ?x ?y) (isa ?y ?z)) -> (inst ?x ?z)").unwrap();

    println!("{}",x);
}
