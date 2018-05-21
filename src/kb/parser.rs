use nom::*;
use std::fs;
use kb::knowledge_base::{Fact, Rule, KnowledgeBase, KB};

pub fn parse_kb_from_file(filename: &str) -> Result<KB, String> {
    let file = fs::read(filename).expect("test.kb not found");

    match kb(&file[..]) {
        Ok(tuple) => Ok(tuple.1),
        Err(_) => Err(String::from("Failed to parse kb from file"))
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

named!(fact<&[u8], Fact>,
    ws!(do_parse!(
        tag!("fact:") >>
        tag!("(") >>
        pred: alpha >>
        args: many1!(map!(name, |c| String::from_utf8(c.to_vec()).unwrap())) >>
        tag!(")") >>
        (Fact {
            pred: String::from_utf8(pred.to_vec()).unwrap(),
            args: args
        })
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

named!(rule<&[u8], Rule>,
    ws!(do_parse!(
        tag!("rule:") >>
        tag!("(") >>
        lhs: many1!(rule_part) >>
        tag!(")") >>
        tag!("->") >>
        rhs: rule_part >>
        (Rule {
            lhs: lhs,
            rhs: rhs
        })
    ))
);

named!(pub kb<&[u8], KB>,
    ws!(do_parse!(
        tag!("kb") >>
        tag!("{") >>
        facts: many1!(fact) >>
        rules: many1!(rule) >>
        (KB {
            facts,
            rules: rules
        })
    ))
);

#[cfg(test)]
mod parse_tests {
    use super::{fact, kb, rule, Fact, Rule, KB, parse_kb_from_file};

    #[test]
    fn parse_fact() {
        assert_eq!(
            fact(&b"fact: (isa cube box)eol"[..]),
            Ok((
                &b"eol"[..],
                Fact {
                    pred: String::from("isa"),
                    args: vec!["cube", "box"].into_iter().map(|w| String::from(w)).collect(),
                }
            ))
        );
    }

    #[test]
    fn parse_rule() {
        assert_eq!(
            rule(&b"rule: ((inst ?x ?y) (isa ?y ?z)) -> (inst ?x ?z)eol"[..]),
            Ok((
                &b"eol"[..],
                Rule {
                    lhs: vec![vec!["inst", "?x", "?y"], vec!["isa", "?y", "?z"]].into_iter().map(|lst| lst.into_iter().map(|w| String::from(w)).collect()).collect(),
                    rhs: vec!["inst", "?x", "?z"].into_iter().map(|w| String::from(w)).collect(),
                }
            ))
        )
    }

    #[test]
    fn parse_kb() {
        assert_eq!(
            kb(&b"kb {\nfact: (isa cube box)\nfact: (isa box container)\nrule: ((inst ?x ?y) (isa ?y ?z)) -> (inst ?x ?z)\n}"[..]),
            Ok((
                &b"}"[..],
                KB {
                    facts: vec![Fact {
                        pred: String::from("isa"),
                        args: vec!["cube","box"].into_iter().map(|w| String::from(w)).collect()
                    }, Fact {
                        pred: String::from("isa"),
                        args: vec!["box","container"].into_iter().map(|w| String::from(w)).collect()
                    }],
                    rules: vec![Rule {
                        lhs: vec![vec!["inst","?x","?y"],vec!["isa","?y","?z"]].into_iter().map(|lst| lst.into_iter().map(|w| String::from(w)).collect()).collect(),
                        rhs: vec!["inst", "?x", "?z"].into_iter().map(|w| String::from(w)).collect()
                    }]
                }
            ))
        )
    }

    #[test]
    fn parse_from_file() {
        assert_eq!(
            parse_kb_from_file("src/test.kb"),
            Ok(KB {
                facts: vec![Fact {
                    pred: String::from("isa"),
                    args: vec!["cube","box"].into_iter().map(|w| String::from(w)).collect()
                }, Fact {
                    pred: String::from("isa"),
                    args: vec!["box","container"].into_iter().map(|w| String::from(w)).collect()
                }],
                rules: vec![Rule {
                    lhs: vec![vec!["inst","?x","?y"],vec!["isa","?y","?z"]].into_iter().map(|lst| lst.into_iter().map(|w| String::from(w)).collect()).collect(),
                    rhs: vec!["inst", "?x", "?z"].into_iter().map(|w| String::from(w)).collect()
                }]
            })
        )
    }
}
