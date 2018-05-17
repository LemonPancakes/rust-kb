extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::*;

#[cfg(debug_assertions)]
const _GRAMMAR: &'static str = include_str!("grammar.pest");

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct PDDLParser;



fn main() {
    let x = PDDLParser::parse(Rule::domain,"(define (Domain testing) \
    (:requirements :strips :equality)\
    (:types ad bd cd - dd)\
    (:constants ad bd cd - dd)\
    (:predicates (PREDICATE_1_NAME ?A1 ?A2 ?AN) (PREDICATE_2_NAME ?A1 ?A2 ?AN)))\
    (:functions (distance ?from ?to) (total-cost))\
    (:constraints (and (always (clean truck1)) (always (clean truck1)) (at end (at package2 paris))))\
    ").unwrap();

    println!("{}",x);
}
