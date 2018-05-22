

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
    Rule
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

impl KnowledgeBase {
    pub fn new(facts: Vec<Fact>, rules: Vec<Rule>) -> KnowledgeBase {
        KnowledgeBase { facts, rules }
    }

    pub fn from(pkb: ParsedKnowledgeBase) -> KnowledgeBase {
        // any post-processing like organizing into HashMap or whatever
        // can be done here
        KnowledgeBase::new(pkb.facts, pkb.rules)
    }

    pub fn ask(&self) -> Result<bool, String> {
        Ok(false)
    }

    pub fn add_fact(&mut self, fact: Fact) -> Result<(), String> {
        Ok(())
    }

    pub fn remove_fact(&mut self, fact: Fact) -> Result<(), String> {
        Ok(())
    }

    pub fn add_rule(&mut self, rule: Rule) -> Result<(), String> {
        Ok(())
    }

    pub fn remove_rule(&mut self, rule: Rule) -> Result<(), String> {
        Ok(())
    }

    pub fn assert<T: Statement>(&mut self, statement: T) -> Result<(), String> {
        Ok(())
    }

    pub fn retract<T: Statement>(&mut self, statement: T) -> Result<(), String> {
        Ok(())
    }
}
