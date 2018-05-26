use kb::parser::{parse_kb_from_file, ParsedFact, ParsedKnowledgeBase, ParsedRule};
use kb::symbols::{Symbol, SymbolTable};

// TODO Eventually maybe(?) want to use these structs
// (Argument, Predicate) to avoid
// having so many copies of the same data
// If not, just delete.
#[derive(Debug, PartialEq, Clone)]
pub struct Argument {
    pub name: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Predicate {
    pub name: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Fact {
    pub pred: Symbol,
    pub args: Vec<Symbol>,
}

impl Fact {
    pub fn new(pred: Symbol, args: Vec<Symbol>) -> Fact {
        Fact { pred, args }
    }

    pub fn from(pf: &ParsedFact, symbols: &mut SymbolTable) -> Fact {
        let pred = symbols.intern(&pf.pred);
        let mut args = Vec::new();
        for parg in pf.args.iter() {
            args.push(symbols.intern(&parg));
        }

        Fact::new(pred, args)
    }

    pub fn from_raw(raw_fact: &Vec<String>, symbols: &mut SymbolTable) -> Fact {
        let mut args = Vec::new();
        let mut pred = symbols.intern("");
        for (i, item) in raw_fact.iter().enumerate() {
            if i == 0 {
                pred = symbols.intern(&item);
            } else {
                args.push(symbols.intern(&item));
            }
        }

        Fact::new(pred, args)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Rule {
    pub lhs: Vec<Fact>,
    pub rhs: Fact,
}

impl Rule {
    pub fn new(lhs: Vec<Fact>, rhs: Fact) -> Rule {
        Rule { lhs, rhs }
    }

    pub fn from(pr: &ParsedRule, symbols: &mut SymbolTable) -> Rule {
        let mut lhs = Vec::new();

        for parsed_raw_fact in pr.lhs.iter() {
            // TODO use Fact's from_raw fn instead
            let mut args = Vec::new();
            let mut pred = symbols.intern("");
            for (i, item) in parsed_raw_fact.iter().enumerate() {
                if i == 0 {
                    pred = symbols.intern(&item);
                } else {
                    args.push(symbols.intern(&item));
                }
            }
            lhs.push(Fact::new(pred, args));
        }

        // TODO use Fact's from_raw fn instead
        let mut args = Vec::new();
        let mut pred = symbols.intern("");
        for (i, item) in pr.rhs.iter().enumerate() {
            if i == 0 {
                pred = symbols.intern(&item);
            } else {
                args.push(symbols.intern(&item));
            }
        }
        let rhs = Fact::new(pred, args);

        Rule::new(lhs, rhs)
    }
}

pub trait Statement {
    fn to_fact(self) -> Option<Fact>;
    fn to_rule(self) -> Option<Rule>;
}
impl Statement for Fact {
    fn to_fact(self) -> Option<Fact> {
        Some(self)
    }
    fn to_rule(self) -> Option<Rule> {
        None
    }
}
impl Statement for Rule {
    fn to_fact(self) -> Option<Fact> {
        None
    }
    fn to_rule(self) -> Option<Rule> {
        Some(self)
    }
}

#[derive(Debug)]
pub struct KnowledgeBase {
    pub facts: Vec<Fact>, // TODO HashMap of preds to arguments
    pub rules: Vec<Rule>,
    pub symbols: SymbolTable, // TODO change to private eventually
}

impl PartialEq for KnowledgeBase {
    fn eq(&self, other: &KnowledgeBase) -> bool {
        self.facts == other.facts && self.rules == other.rules
    }
}

//TODO most of these functions will need to be reimplemented
// based on new KnowledgeBase data structure
impl KnowledgeBase {
    pub fn new(facts: Vec<Fact>, rules: Vec<Rule>, symbols: SymbolTable) -> KnowledgeBase {
        KnowledgeBase {
            facts,
            rules,
            symbols,
        }
    }

    pub fn from(pkb: ParsedKnowledgeBase) -> KnowledgeBase {
        let mut facts = Vec::new();
        let mut rules = Vec::new();
        let mut symbols = SymbolTable::new();

        for parsed_fact in pkb.facts.iter() {
            facts.push(Fact::from(&parsed_fact, &mut symbols));
        }

        for parsed_rule in pkb.rules.iter() {
            rules.push(Rule::from(&parsed_rule, &mut symbols));
        }

        KnowledgeBase::new(facts, rules, symbols)
    }

    pub fn from_file(filename: &str) -> Result<KnowledgeBase, String> {
        let pkb = parse_kb_from_file(filename)?;
        Ok(KnowledgeBase::from(pkb))
    }

    pub fn intern_string(&mut self, name: &str) -> Symbol {
        self.symbols.intern(name)
    }

    // TODO do inference;
    pub fn assert<T: Statement + Copy>(&mut self, statement: T) -> Result<(), String> {
        match statement.to_fact() {
            Some(fact) => {
                return self.add_fact(fact);
            }
            None => {
                let rule = statement.to_rule().unwrap();
                return self.add_rule(rule);
            }
        }
    }

    pub fn retract<T: Statement + Copy>(&mut self, statement: T) -> Result<(), String> {
        match statement.to_fact() {
            Some(fact) => {
                return self.remove_fact(&fact);
            }
            None => {
                let rule = statement.to_rule().unwrap();
                return self.remove_rule(&rule);
            }
        }
    }

    pub fn ask(&self, fact: &Fact) -> Result<bool, String> {
        if self.contains_fact(fact) {
            return Ok(true);
        }
        Ok(false)
    }

    fn add_fact(&mut self, fact: Fact) -> Result<(), String> {
        if self.contains_fact(&fact) {
            return Err(String::from("fact already in kb"));
        }
        self.facts.push(fact);
        Ok(())
    }

    fn remove_fact(&mut self, fact: &Fact) -> Result<(), String> {
        if !self.contains_fact(fact) {
            return Err(String::from("fact does not exist in kb"));
        }
        let index = self.facts.iter().position(|x| *x == *fact).unwrap();
        self.facts.remove(index);
        Ok(())
    }

    fn add_rule(&mut self, rule: Rule) -> Result<(), String> {
        if self.contains_rule(&rule) {
            return Err(String::from("rule already in kb"));
        }
        self.rules.push(rule);
        Ok(())
    }

    fn remove_rule(&mut self, rule: &Rule) -> Result<(), String> {
        if !self.contains_rule(rule) {
            return Err(String::from("rule does not exist in kb"));
        }
        let index = self.rules.iter().position(|x| *x == *rule).unwrap();
        self.rules.remove(index);
        Ok(())
    }

    fn contains_fact(&self, fact: &Fact) -> bool {
        self.facts.contains(fact)
    }

    fn contains_rule(&self, rule: &Rule) -> bool {
        self.rules.contains(rule)
    }
}

#[allow(unused_must_use)]
#[cfg(test)]
mod knowledge_base_tests {
    use super::*;

    #[test]
    fn test_add_fact() {
        let mut kb = KnowledgeBase::new(vec![], vec![], SymbolTable::new());
        let new_fact = Fact::new(
            kb.intern_string("isa"),
            vec![kb.intern_string("Bob"), kb.intern_string("boy")],
        );
        kb.add_fact(new_fact.clone());

        assert_eq!(kb.contains_fact(&new_fact), true);
    }

    #[test]
    fn test_remove_fact() {
        let mut st = SymbolTable::new();
        let new_fact = Fact::new(st.intern("isa"), vec![st.intern("Bob"), st.intern("boy")]);
        let mut kb = KnowledgeBase::new(vec![new_fact.clone()], vec![], st);
        kb.remove_fact(&new_fact);

        assert_eq!(kb.contains_fact(&new_fact), false);
        assert_eq!(kb.facts.is_empty(), true);
    }

    #[test]
    fn test_ask_fact_already_in_kb() {
        let mut st = SymbolTable::new();
        let new_fact = Fact::new(st.intern("isa"), vec![st.intern("Bob"), st.intern("boy")]);
        let kb = KnowledgeBase::new(vec![new_fact.clone()], vec![], st);
        assert_eq!(kb.ask(&new_fact), Ok(true))
    }

    #[test]
    fn test_ask_fact_not_in_fb() {
        let mut st = SymbolTable::new();
        let new_fact = Fact::new(st.intern("isa"), vec![st.intern("Bob"), st.intern("boy")]);
        let kb = KnowledgeBase::new(vec![], vec![], st);
        assert_eq!(kb.ask(&new_fact), Ok(false));
    }

    #[test]
    fn test_ask_fact_inferred_from_rule_in_kb() {}
}
