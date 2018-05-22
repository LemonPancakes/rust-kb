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

#[derive(Debug, PartialEq, Clone)]
pub struct Fact {
    pub pred: String,      // TODO Rc<Predicate> (maybe?)
    pub args: Vec<String>, // TODO Vec<Rc<Argument>> (maybe?)
}

impl Fact {
    pub fn new(pred: String, args: Vec<String>) -> Fact {
        Fact { pred, args }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Rule {
    pub lhs: Vec<Vec<String>>,
    pub rhs: Vec<String>,
}

impl Rule {
    pub fn new(lhs: Vec<Vec<String>>, rhs: Vec<String>) -> Rule {
        Rule { lhs, rhs }
    }
}

// TODO please tell me there is a better way to do this...!
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
    pub facts: Vec<Fact>, // TODO HashMap<Rc<Predicate>, Vec<Rc<Argument>>> (maybe?)
    // at least should be HashMap<String, Vec<String>>
    pub rules: Vec<Rule>,
}

//TODO most of these functions will need to be reimplemented
// based on new KnowledgeBase data structure
#[allow(dead_code)]
impl KnowledgeBase {
    pub fn new(facts: Vec<Fact>, rules: Vec<Rule>) -> KnowledgeBase {
        KnowledgeBase { facts, rules }
    }

    pub fn from(pkb: ParsedKnowledgeBase) -> KnowledgeBase {
        // any post-processing like organizing into HashMap or whatever
        // can be done here
        KnowledgeBase::new(pkb.facts, pkb.rules)
    }

    // TODO I couldn't figure out a good way to check if statements are Rules or Facts
    // then cast and pass them:
    pub fn assert<T: Statement>(&mut self, statement: T) -> Result<(), String> {
        Ok(())
    }

    pub fn retract<T: Statement>(&mut self, statement: T) -> Result<(), String> {
        Ok(())
    }

    pub fn ask(&self, fact: &Fact) -> Result<bool, String> {
        if self.contains_fact(fact) {
            return Ok(true);
        }

        // TODO missing inference for rules

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

#[allow(unused_must_use)]
#[cfg(test)]
mod knowledge_base_tests {
    use super::*;

    #[test]
    fn test_add_fact() {
        let mut kb = KnowledgeBase::new(vec![], vec![]);
        let new_fact = Fact::new(
            "isa".to_string(),
            vec!["Bob".to_string(), "boy".to_string()],
        );
        kb.add_fact(new_fact.clone());

        assert_eq!(kb.contains_fact(&new_fact), true);
    }

    #[test]
    fn test_remove_fact() {
        let new_fact = Fact::new(
            "isa".to_string(),
            vec!["Bob".to_string(), "boy".to_string()],
        );
        let mut kb = KnowledgeBase::new(vec![new_fact.clone()], vec![]);
        kb.remove_fact(&new_fact);

        assert_eq!(kb.contains_fact(&new_fact), false);
        assert_eq!(kb.facts.is_empty(), true);
    }

    #[test]
    fn test_ask_fact_already_in_kb() {
        let new_fact = Fact::new(
            "isa".to_string(),
            vec!["Bob".to_string(), "boy".to_string()],
        );
        let kb = KnowledgeBase::new(vec![new_fact.clone()], vec![]);
        assert_eq!(kb.ask(&new_fact), Ok(true))
    }

    #[test]
    fn test_ask_fact_not_in_fb() {
        let new_fact = Fact::new(
            "isa".to_string(),
            vec!["Bob".to_string(), "boy".to_string()],
        );
        let kb = KnowledgeBase::new(vec![], vec![]);
        assert_eq!(kb.ask(&new_fact), Ok(false));
    }

    #[test]
    fn test_ask_fact_inferred_from_rule_in_kb() {}
}
