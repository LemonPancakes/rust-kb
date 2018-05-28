//extern crate pest;
//#[macro_use]
//extern crate pest_derive;

use pest::*;

#[cfg(debug_assertions)]
const _GRAMMAR: &'static str = include_str!("kb.pest");

#[derive(Parser)]
#[grammar = "kb.pest"]
pub struct PDDLParser;

pub fn parse<'a>(file: &'a str) -> iterators::Pairs<'a, Rule> {
    PDDLParser::parse(Rule::kb, file).unwrap()
}
