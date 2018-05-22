// TODO First parse into this struct, then post process
// data into KnowledgeBase. Or maybe we don't need
// this at all and we can parse directly into KnowledgeBase
#[derive(Debug, PartialEq)]
pub struct ParsedKnowledgeBase {
    pub facts: Vec<Fact>,
    pub rules: Vec<Rule>,
}

// TODO Eventually maybe(?) want to use these structs
// (Argument, Predicate) to avoid
// having so many copies of the same data
// If not, just delete.
#[derive(Debug, PartialEq)]
pub struct Argument {
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct Predicate {
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct Fact {
    pub pred: String,      // TODO Rc<Predicate> (?)
    pub args: Vec<String>, // TODO Vec<Rc<Argument>> (?)
}

#[derive(Debug, PartialEq)]
pub struct Rule {
    pub lhs: Vec<Vec<String>>,
    pub rhs: Vec<String>,
}

// please tell me there is a better way to do this...
pub enum StatementType {
    Fact,
    Rule,
}
pub trait Statement {
    fn stype(&self) -> StatementType;
}
impl Statement for Fact {
    fn stype(&self) -> StatementType {
        StatementType::Fact
    }
}
impl Statement for Rule {
    fn stype(&self) -> StatementType {
        StatementType::Rule
    }
}

#[derive(Debug, PartialEq)]
pub struct KnowledgeBase {
    pub facts: Vec<Fact>, // TODO HashMap<Rc<Predicate>, Vec<Rc<Argument>>> (?)
    pub rules: Vec<Rule>,
}

// most of these functions will need to be reimplemented
// based on new KnowledgeBase data structure
impl KnowledgeBase {
    pub fn new(facts: Vec<Fact>, rules: Vec<Rule>) -> KnowledgeBase {
        KnowledgeBase { facts, rules }
    }

    pub fn from(pkb: ParsedKnowledgeBase) -> KnowledgeBase {
        // any post-processing like organizing into HashMap or whatever
        // can be done here
        KnowledgeBase::new(pkb.facts, pkb.rules)
    }

    pub fn assert<T: Statement>(&mut self, statement: T) -> Result<(), String> {
        Ok(())
    }

    pub fn retract<T: Statement>(&mut self, statement: T) -> Result<(), String> {
        Ok(())
    }

    pub fn ask<T>(&self, statement: T) -> Result<bool, String> {
        Ok(false)
    }

    fn add_fact(&mut self, fact: Fact) -> Result<(), String> {
        self.facts.push(fact);
        Ok(())
    }

    fn remove_fact(&mut self, fact: &Fact) -> Result<(), String> {
        let index = self.facts.iter().position(|x| *x == *fact).unwrap();
        self.facts.remove(index);
        Ok(())
    }

    fn add_rule(&mut self, rule: Rule) -> Result<(), String> {
        self.rules.push(rule);
        Ok(())
    }

    fn remove_rule(&mut self, rule: Rule) -> Result<(), String> {
        let index = self.rules.iter().position(|x| *x == rule).unwrap();
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
