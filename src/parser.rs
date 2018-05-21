use nom::*;
use std::str;

use knowledge_base::*;

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
        args: many1!(map!(name, |c| str::from_utf8(c).unwrap())) >>
        tag!(")") >>
        (Fact {
            pred: str::from_utf8(pred).unwrap(),
            args: args
        })
    ))
);

named!(rule_part<&[u8], Vec<&str> >,
    ws!(do_parse!(
        tag!("(") >>
        args: many1!(map!(alt!(name | var), |c| str::from_utf8(c).unwrap())) >>
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
        facts: many1!(fact) >>
        rules: many1!(rule) >>
        (KB {
            facts,
            rules
        })
    ))
);

#[cfg(test)]
mod parse_tests {
    use super::{fact, kb, rule, Fact, Rule, KB};

    #[test]
    fn parse_fact() {
        assert_eq!(
            fact(&b"fact: (isa cube box)eol"[..]),
            Ok((
                &b"eol"[..],
                Fact {
                    pred: "isa",
                    args: vec!["cube", "box"],
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
                    lhs: vec![vec!["inst", "?x", "?y"], vec!["isa", "?y", "?z"]],
                    rhs: vec!["inst", "?x", "?z"],
                }
            ))
        )
    }

    #[test]
    fn parse_kb() {
        assert_eq!(
            kb(&b"fact: (isa cube box)\nfact: (isa box container)\nrule: ((inst ?x ?y) (isa ?y ?z)) -> (inst ?x ?z)eol"[..]),
            Ok((
                &b"eol"[..],
                KB {
                    facts: vec![Fact {
                        pred: "isa",
                        args: vec!["cube","box"]
                    }, Fact {
                        pred: "isa",
                        args: vec!["box","container"]
                    }],
                    rules: vec![Rule {
                        lhs: vec![vec!["inst","?x","?y"],vec!["isa","?y","?z"]],
                        rhs: vec!["inst", "?x", "?z"]
                    }]
                }
            ))
        )
    }
}
