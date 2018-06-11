#![allow(dead_code)]

use nom::*;
use std::fs;

#[derive(Debug, PartialEq)]
pub struct ParsedKnowledgeBase {
    pub statements: Vec<ParsedStatement>
}

impl ParsedKnowledgeBase {
    pub fn new(statements: Vec<ParsedStatement>) -> ParsedKnowledgeBase {
        ParsedKnowledgeBase { statements }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParsedStatement {
    Fact(ParsedFact),
    Rule(ParsedRule),
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

pub fn parse_fact(f: &[u8]) -> Result<ParsedFact, String> {
    match fact(f) {
        Ok(tuple) => match tuple.1 {
            ParsedStatement::Fact(parsed_fact) => Ok(parsed_fact),

            _ => Err(String::from("Failed to parse fact from string")),
        }
        Err(_) => Err(String::from("Failed to parse fact from string")),
    }
}

pub fn parse_rule(r: &[u8]) -> Result<ParsedRule, String> {
    match rule(r) {
        Ok(tuple) => match tuple.1 {
            ParsedStatement::Rule(parsed_rule) => Ok(parsed_rule),

            _ => Err(String::from("Failed to parse rule from string")),
        }
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

named!(fact<&[u8], ParsedStatement>,
    do_parse!(
    fact : ws!(do_parse!(
        pred : alpha >>
        tag!("(") >>
        args: many1!(map!(do_parse!(
                              arg : alt!(name | var) >>
                              opt!(tag!(",")) >>
                              (arg)), |c| String::from_utf8(c.to_vec()).unwrap())) >>
        tag!(")") >>
        (ParsedStatement::Fact(ParsedFact::new(String::from_utf8(pred.to_vec()).unwrap(), args)))
    )) >>
    tag!(".") >>
    (fact))
);

named!(rule_part<&[u8], Vec<String> >,
    do_parse!(
    part : ws!(do_parse!(
        pred : alpha >>
        tag!("(") >>
        args: many1!(map!(do_parse!(
                              arg : alt!(name | var) >>
                              opt!(tag!(",")) >>
                              (arg)), |c| String::from_utf8(c.to_vec()).unwrap())) >>
        tag!(")") >>
        ({ let mut p = vec![String::from_utf8(pred.to_vec()).unwrap()];
            for a in args {
                p.push(a);
            }
            p
            })))
 >>
    (part))
);

named!(rule<&[u8], ParsedStatement>,
    do_parse!(
        rule: ws!(do_parse!(
        rhs: rule_part >>
        tag!(":-") >>
        lhs: many1!(do_parse!(
                        part : rule_part >>
                        opt!(tag!(",")) >>
                        (part))) >>
        (ParsedStatement::Rule(ParsedRule::new(lhs, rhs)))
    )) >>
    tag!(".") >>
    (rule))
);

named!(kb<&[u8], ParsedKnowledgeBase>,
    do_parse!( pkb : ws!(do_parse!(
        statements: many1!(alt!(fact | rule)) >>
        (ParsedKnowledgeBase::new(statements))
    )) >>
    tag!(".") >>
    (pkb))
);

#[cfg(test)]
mod parse_tests {
    use super::*;

    #[test]
    fn parse_fact() {
        assert_eq!(
            fact(&b"isa(cube,box)."[..]),
            Ok((
                &b""[..],
                ParsedStatement::Fact(ParsedFact::new(
                    String::from("isa"),
                    vec!["cube", "box"]
                        .into_iter()
                        .map(|w| String::from(w))
                        .collect(),
                ))
            ))
        );
    }

    #[test]
    fn parse_rule() {
        assert_eq!(
            rule(&b"inst(?x, ?z) :- inst(?x, ?y), isa(?y, ?z)."[..]),
            Ok((
                &b""[..],
                ParsedStatement::Rule(ParsedRule::new(
                    vec![vec!["inst", "?x", "?y"], vec!["isa", "?y", "?z"]]
                        .into_iter()
                        .map(|lst| lst.into_iter().map(|w| String::from(w)).collect())
                        .collect(),
                    vec!["inst", "?x", "?z"]
                        .into_iter()
                        .map(|w| String::from(w))
                        .collect(),
                ))
            ))
        )
    }

    #[test]
    fn parse_kb() {
        assert_eq!(
            kb(&b"isa(cube, box). isa(box, container). inst(?x, ?z) :- inst(?x, ?y), isa(?y ?z).."[..]),
            Ok((
                &b""[..],
                ParsedKnowledgeBase {
                    statements: vec![ParsedStatement::Fact(ParsedFact::new(
                        String::from("isa"),
                        vec!["cube", "box"].into_iter().map(|w| String::from(w)).collect()
                    )),
                                     ParsedStatement::Fact(ParsedFact::new(
                                         String::from("isa"),
                                         vec!["box", "container"].into_iter().map(|w| String::from(w)).collect()
                                     )),
                                     ParsedStatement::Rule(ParsedRule::new(
                                         vec![vec!["inst", "?x", "?y"], vec!["isa", "?y", "?z"]].into_iter().map(|lst| lst.into_iter().map(|w| String::from(w)).collect()).collect(),
                                         vec!["inst", "?x", "?z"].into_iter().map(|w| String::from(w)).collect()
                                     ))]
                }
            ))
        )
    }

    #[test]
    fn parse_from_file() {
        assert_eq!(
            parse_kb_from_file("test/test.kb"),
            Ok(ParsedKnowledgeBase::new(
                 vec![ParsedStatement::Fact(ParsedFact::new(
            String::from("isa"),
            vec!["cube", "box"]
                .into_iter()
                .map(|w| String::from(w))
                .collect(),
        )),
            ParsedStatement::Fact(ParsedFact::new(
                String::from("isa"),
                vec!["box", "container"]
                    .into_iter()
                    .map(|w| String::from(w))
                    .collect(),
            )),
            ParsedStatement::Rule(ParsedRule::new(
                vec![vec!["inst", "?x", "?y"], vec!["isa", "?y", "?z"]]
                    .into_iter()
                    .map(|lst| lst.into_iter().map(|w| String::from(w)).collect())
                    .collect(),
                vec!["inst", "?x", "?z"]
                    .into_iter()
                    .map(|w| String::from(w))
                    .collect(),
            ))])));
    }
}
