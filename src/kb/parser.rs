#![allow(dead_code)]

use nom::*;
use std::fs;


#[derive(Debug, PartialEq)]
pub struct ParsedKnowledgeBase {
    pub facts: Vec<ParsedFact>,
    pub rules: Vec<ParsedRule>,
}

impl ParsedKnowledgeBase {
    pub fn new(facts: Vec<ParsedFact>, rules: Vec<ParsedRule>) -> ParsedKnowledgeBase {
        ParsedKnowledgeBase { facts, rules }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ParsedFact {
    pub pred: String,
    pub args: Vec<String>,
}

impl ParsedFact {
    pub fn new(pred: String, args: Vec<String>) -> ParsedFact {
        ParsedFact { pred, args }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ParsedRule {
    pub lhs: Vec<Vec<String>>,
    pub rhs: Vec<String>,
}

impl ParsedRule {
    pub fn new(lhs: Vec<Vec<String>>, rhs: Vec<String>) -> ParsedRule {
        ParsedRule { lhs, rhs }
    }
}

pub fn parse_kb_from_file(filename: &str) -> Result<ParsedKnowledgeBase, String> {
    let file = fs::read(filename).expect("file not found");

    match kb(&file[..]) {
        Ok(tuple) => Ok(tuple.1),
        Err(_) => Err(String::from("Failed to parse kb from file")),
    }
}

pub fn parse_fact(f : &[u8]) -> Result<ParsedFact, String> {
    match fact(f) {
        Ok(tuple) => Ok(tuple.1),
        Err(_) => Err(String::from("Failed to parse fact from string")),
    }
}

pub fn parse_rule(r : &[u8]) -> Result<ParsedRule, String> {
    match rule(r) {
        Ok(tuple) => Ok(tuple.1),
        Err(_) => Err(String::from("Failed to parse rule from string")),
    }
}

named!(
    name<&[u8]>,
    recognize!(pair!(
        take_while1!(is_alphabetic),
        take_while!(is_alphanumeric)
    ))
);

named!(
    var<&[u8]>,
    recognize!(pair!(
        take_while1!(|c| c == b'?'),
        take_while!(is_alphanumeric)
    ))
);

named!(fact<&[u8], ParsedFact>,
    ws!(do_parse!(
        tag!("fact:") >>
        tag!("(") >>
        pred: alpha >>
        args: many1!(map!(alt!(name | var), |c| String::from_utf8(c.to_vec()).unwrap())) >>
        tag!(")") >>
        (ParsedFact::new(String::from_utf8(pred.to_vec()).unwrap(), args))
    ))
);

named!(rule_part<&[u8], Vec<String> >,
    ws!(do_parse!(
        tag!("(") >>
        args: many1!(map!(alt!(name | var), |c| String::from_utf8(c.to_vec()).unwrap())) >>
        tag!(")") >>
        (args)
    ))
);

named!(rule<&[u8], ParsedRule>,
    ws!(do_parse!(
        tag!("rule:") >>
        tag!("(") >>
        lhs: many1!(rule_part) >>
        tag!(")") >>
        tag!("->") >>
        rhs: rule_part >>
        (ParsedRule::new(lhs, rhs))
    ))
);

named!(kb<&[u8], ParsedKnowledgeBase>,
    ws!(do_parse!(
        tag!("kb") >>
        tag!("{") >>
        facts: many1!(fact) >>
        rules: many1!(rule) >>
        (ParsedKnowledgeBase { facts, rules })
    ))
);

#[cfg(test)]
mod parse_tests {
    use super::*;

    #[test]
    fn parse_fact() {
        assert_eq!(
            fact(&b"fact: (isa cube box)eol"[..]),
            Ok((
                &b"eol"[..],
                ParsedFact::new(
                    String::from("isa"),
                    vec!["cube", "box"]
                        .into_iter()
                        .map(|w| String::from(w))
                        .collect(),
                )
            ))
        );
    }

    #[test]
    fn parse_rule() {
        assert_eq!(
            rule(&b"rule: ((inst ?x ?y) (isa ?y ?z)) -> (inst ?x ?z)eol"[..]),
            Ok((
                &b"eol"[..],
                ParsedRule::new(
                    vec![vec!["inst", "?x", "?y"], vec!["isa", "?y", "?z"]]
                        .into_iter()
                        .map(|lst| lst.into_iter().map(|w| String::from(w)).collect())
                        .collect(),
                    vec!["inst", "?x", "?z"]
                        .into_iter()
                        .map(|w| String::from(w))
                        .collect(),
                )
            ))
        )
    }

    #[test]
    fn parse_kb() {
        assert_eq!(
            kb(&b"kb {\nfact: (isa cube box)\nfact: (isa box container)\nrule: ((inst ?x ?y) (isa ?y ?z)) -> (inst ?x ?z)\n}"[..]),
            Ok((
                &b"}"[..],
                ParsedKnowledgeBase {
                    facts: vec![ParsedFact::new(
                        String::from("isa"),
                        vec!["cube","box"].into_iter().map(|w| String::from(w)).collect()
                    ), ParsedFact::new(
                        String::from("isa"),
                        vec!["box","container"].into_iter().map(|w| String::from(w)).collect()
                    )],
                    rules: vec![ParsedRule::new(
                        vec![vec!["inst","?x","?y"],vec!["isa","?y","?z"]].into_iter().map(|lst| lst.into_iter().map(|w| String::from(w)).collect()).collect(),
                        vec!["inst", "?x", "?z"].into_iter().map(|w| String::from(w)).collect()
                    )]
                }
            ))
        )
    }

    #[test]
    fn parse_from_file() {
        assert_eq!(
            parse_kb_from_file("test/test.kb"),
            Ok(ParsedKnowledgeBase::new(
                vec![
                    ParsedFact::new(
                        String::from("isa"),
                        vec!["cube", "box"]
                            .into_iter()
                            .map(|w| String::from(w))
                            .collect(),
                    ),
                    ParsedFact::new(
                        String::from("isa"),
                        vec!["box", "container"]
                            .into_iter()
                            .map(|w| String::from(w))
                            .collect(),
                    ),
                ],
                vec![
                    ParsedRule::new(
                        vec![vec!["inst", "?x", "?y"], vec!["isa", "?y", "?z"]]
                            .into_iter()
                            .map(|lst| lst.into_iter().map(|w| String::from(w)).collect())
                            .collect(),
                        vec!["inst", "?x", "?z"]
                            .into_iter()
                            .map(|w| String::from(w))
                            .collect(),
                    ),
                ],
            ))
        )
    }
}
