#![allow(dead_code)]

use kb::parser::{parse_kb_from_file, ParsedFact, ParsedKnowledgeBase, ParsedRule};
use kb::symbols::{Symbol, SymbolTable};

use std::collections::{HashMap,HashSet};
use std::rc::Rc;

// TODO Eventually maybe(?) want to use these structs
// (Argument, Predicate) to avoid
// having so many copies of the same data
// If not, just delete.
//#[derive(Debug, PartialEq, Clone)]
//pub struct Argument {
//    pub name: String,
//}
//
//#[derive(Debug, PartialEq, Clone)]
//pub struct Predicate {
//    pub name: String,
//}

/// Defines a fact relationship between two or more arguments
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
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

/// Defines a knowledge base fact that can be infered, given 1 or more facts as the premise
///
/// Allows structures and complex dependencies to be imposed on a knowledge base without the
/// need for every rule to be defined explictly. When a rule is added to a knowledge base,
/// it will use logical inference by forward chaining to automatically create the implied rules.
/// Use this if you want to impose specialized relationships that are not the default assumption
/// for rules.
#[derive(Debug, PartialEq, Clone)]
pub struct Rule {
    pub lhs: Vec<Fact>,
    pub rhs: Fact,
}

impl Rule {
    /// Create a new rule from Facts
    pub fn new(lhs: Vec<Fact>, rhs: Fact) -> Rule {
        Rule { lhs, rhs }
    }

    /// Create a new rule from a parsed object
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

/// Abstraction that combines facts and rules
///
/// This trait allows objects to be either facts and rules, and then do case handling depending
/// on the specific statement passed in
pub trait Statement {
    fn to_fact(&self) -> Option<Fact>;
    fn to_rule(&self) -> Option<Rule>;
}

impl Statement for Fact {
    fn to_fact(&self) -> Option<Fact> {
        Some(self.clone())
    }
    fn to_rule(&self) -> Option<Rule> {
        None
    }
}

impl Statement for Rule {
    fn to_fact(&self) -> Option<Fact> {
        None
    }
    fn to_rule(&self) -> Option<Rule> {
        Some(self.clone())
    }
}

// We added these type aliases to make long types more readable
// We are unsure if these are good/if there is a better way
// It makes the types easier to read, but makes them more opaque
type ArgumentHash = HashMap<Symbol, Vec<Rc<Fact>>>;
type QueryBinding = Vec<(Symbol,Symbol)>;

/// A data structure which can take in facts and rules, and respond to logical questions and queries
///
/// A knowledge base can take (as well as remove) facts and rules to generate facts which it
/// knows to be true. Then, the knowledge base can be asked logical questions and will answer whether
/// it can determine the answer.
#[derive(Debug)]
pub struct KnowledgeBase {
    facts: Vec<Rc<Fact>>,
    facts_map: HashMap<Symbol, Vec<ArgumentHash>>,

    rules: Vec<Rule>,
    symbols: SymbolTable, // TODO change to private eventually
}

impl PartialEq for KnowledgeBase {
    fn eq(&self, other: &KnowledgeBase) -> bool {
        //TODO Make this work as expected. AKA not depend on order of insertion
        self.facts == other.facts && self.rules == other.rules
    }
}

impl KnowledgeBase {
    /// Creates a new knowledge base with given rules and facts and a symbol table
    pub fn new(facts: Vec<Fact>, rules: Vec<Rule>, symbols: SymbolTable) -> KnowledgeBase {
        let mut kb = KnowledgeBase {
            facts: Vec::new(),
            facts_map: HashMap::new(),
            rules,
            symbols,
        };

        for fact in facts {
            kb.insert_fact(fact);
        }

        kb
    }

    /// Creates a knowledge base from a parsed object from the crate's parser
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

    /// Attempts to create a knowledge base from a given input file
    pub fn from_file(filename: &str) -> Result<KnowledgeBase, String> {
        let pkb = parse_kb_from_file(filename)?;
        Ok(KnowledgeBase::from(pkb))
    }

    pub fn intern_string(&mut self, name: &str) -> Symbol {
        self.symbols.intern(name)
    }

    /// Add a fact or rule to the knowledge base
    ///
    /// This function will use inference by forward chaining to add implied facts from given rules.
    pub fn assert<T: Statement>(&mut self, statement: T) -> Result<(), String> {
        match statement.to_fact() {
            Some(fact) => {
                for rule in self.rules.clone().iter() {
                    self.infer(&fact, &rule);
                }
                return self.add_fact(fact);
            }
            None => {
                let rule = statement.to_rule().unwrap();
                for fact in self.facts.clone().iter() {
                    self.infer(&fact, &rule);
                }
                return self.add_rule(rule);
            }
        }
    }

    /// Remove a fact or rule from the knowledge base
    pub fn retract<T: Statement>(&mut self, statement: T) -> Result<(), String> {
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

    /// Ask if a specific fact can be proven by the knowledge base
    pub fn ask(&self, fact: &Fact) -> Result<bool, String> {
        if self.contains_fact(fact) {
            return Ok(true);
        }
        Ok(false)
    }

    fn insert_fact(&mut self, fact: Fact) {
        let fact_reference = Rc::new(fact);
        self.facts.push(fact_reference.clone());

        let args_vec = self.facts_map
            .entry(fact_reference.pred.clone())
            .or_insert(Vec::new());

        if args_vec.len() == 0 {
            for _ in 0..fact_reference.args.len() {
                args_vec.push(HashMap::new());
            }
        }

        for j in 0..args_vec.len() {
            let mut arg_list = args_vec[j]
                .entry(fact_reference.args[j].clone())
                .or_insert(Vec::new());
            arg_list.push(fact_reference.clone());
        }
    }

    fn add_fact(&mut self, fact: Fact) -> Result<(), String> {
        if self.contains_fact(&fact) {
            return Err(String::from("fact already in kb"));
        }

        self.insert_fact(fact);
        Ok(())
    }

    fn remove_fact(&mut self, fact: &Fact) -> Result<(), String> {
        let mut fact_to_remove = None;

        for i in 0..self.facts.len() {
            if fact == &*self.facts[i] {
                fact_to_remove = Some(self.facts[i].clone());
                self.facts.remove(i);
            }
        }

        match fact_to_remove {
            None => Err(String::from("fact does not exist in kb")),

            Some(fact_reference) => {
                //self.facts.push(fact_reference.clone());

                let mut args_vec = self.facts_map.get_mut(&fact_reference.pred).unwrap(); //.or_insert(Vec::new());

                for j in 0..args_vec.len() {
                    let mut arg_list = args_vec[j].get_mut(&fact_reference.args[j]).unwrap(); //.entry(fact_reference.args[j].clone()).or_insert(Vec::new());
                                                                                              //arg_list.push(fact_reference.clone());

                    let index = arg_list.iter().position(|x| *x == fact_reference).unwrap();
                    arg_list.remove(index);
                }

                Ok(())
            }
        }
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
        self.facts.iter().fold(false, |acc, f| acc || &**f == fact)
    }

    fn contains_rule(&self, rule: &Rule) -> bool {
        self.rules.contains(rule)
    }

    pub fn infer(&mut self, fact: &Fact, rule: &Rule) {
        // Inference by Forward Chaining
        if rule.lhs.len() == 1 {
            let lhs = &rule.lhs[0];
            if let Ok(bindings) = self.try_bind(fact, lhs) {
                let new_fact = self.apply_bindings(&rule.rhs, &bindings);
                if !self.has_var(&new_fact) {
                    assert!(self.assert(new_fact).is_ok());
                }
            }
        } else if rule.lhs.len() > 1 {
            let lhs = &rule.lhs[0];
            if let Ok(bindings) = self.try_bind(fact, lhs) {
                let new_lhs = rule.lhs
                    .clone()
                    .iter()
                    .enumerate()
                    .filter(|&(n, _)| n != 0)
                    .map(|(_, f)| self.apply_bindings(f, &bindings))
                    .collect::<Vec<Fact>>();
                let new_rhs = self.apply_bindings(&rule.rhs, &bindings);
                let new_rule = Rule::new(new_lhs, new_rhs);
                assert!(self.assert(new_rule).is_ok());
            }
        }
    }

    pub fn try_bind(&self, f1: &Fact, f2: &Fact) -> Result<HashMap<Symbol, Symbol>, String> {
        if f1.pred != f2.pred || f1.args.len() != f2.args.len() {
            return Err("bind failed".to_string());
        }
        let mut bindings: HashMap<Symbol, Symbol> = HashMap::new();
        for pairs in f1.args.iter().zip(f2.args.iter()) {
            let (a1, a2) = pairs;
            if a1 != a2 {
                if a1.is_var() && !a2.is_var() {
                    bindings.insert(a1.clone(), a2.clone());
                } else if a2.is_var() && !a1.is_var() {
                    bindings.insert(a2.clone(), a1.clone());
                } else {
                    return Err("bind failed".to_string());
                }
            }
        }
        Ok(bindings)
    }

    pub fn apply_bindings(&self, fact: &Fact, bindings: &HashMap<Symbol, Symbol>) -> Fact {
        let mut args: Vec<Symbol> = Vec::new();
        for symbol in fact.args.iter() {
            if symbol.is_var() {
                if !bindings.contains_key(symbol) {
                    args.push(symbol.clone());
                // return Err("Failed to apply bindings".to_string());
                } else {
                    args.push(bindings.get(symbol).unwrap().clone());
                }
            } else {
                args.push(symbol.clone());
            }
        }
        Fact::new(fact.pred.clone(), args)
    }

    fn has_var(&self, fact: &Fact) -> bool {
        for symbol in fact.args.iter() {
            if symbol.is_var() {
                return true;
            }
        }
        return false;
    }

    /// Query a knowledge base to find all possible bindings to variables in the fact
    pub fn query(&self, f : &Fact) -> Vec<QueryBinding> {
        let mut query_indices = vec![];

        for i in 0..f.args.len() {
            if f.args[i].is_var() {query_indices.push(i);}
        }

        let mut bindings = vec![];

        for matching_fact in self.get_query_facts(f) {
            let mut curr_binding = vec![];
            for i in query_indices.iter() {
                curr_binding.push((f.args[*i].clone(),matching_fact.args[*i].clone()));
            }
            bindings.push(curr_binding);
        }

        bindings
    }

    fn get_query_facts(&self, f : &Fact) -> Vec<Rc<Fact>> {
        match self.facts_map.get(&f.pred) {
            Some(arg_list) => {
                if arg_list.len() == f.args.len() {
                    let mut facts = HashSet::new();
                    let mut any_bind = false;

                    for i in 0..arg_list.len() {
                        if !f.args[i].is_var() {

                            if let Some(fact_list) = arg_list[i].get(&f.args[i]) {
                                let temp_facts = fact_list.iter().map(|f| f.clone()).collect();
                                if any_bind {
                                    facts = facts.intersection(&temp_facts).map(|f| f.clone()).collect();
                                } else {
                                    facts = temp_facts;
                                }
                            }

                            any_bind = true;
                        }
                    }

                    if !any_bind {
                        for fact_list in arg_list[0].values() {
                            for fact in fact_list {
                                facts.insert(fact.clone());
                            }
                        }
                    }
                    return facts.iter().map({ |f| f.clone() }).collect();
                }
            }
            None => {}
        }
        vec![]
    }
}

#[cfg(test)]
mod knowledge_base_basic_tests {
    use super::*;

    #[test]
    fn test_add_fact() {
        let mut kb = KnowledgeBase::new(vec![], vec![], SymbolTable::new());
        let new_fact = Fact::new(
            kb.intern_string("isa"),
            vec![kb.intern_string("Bob"), kb.intern_string("boy")],
        );

        match kb.add_fact(new_fact.clone()) {
            Ok(_) => {}
            Err(e) => println!("{}", e),
        }

        assert_eq!(kb.contains_fact(&new_fact), true);
    }

    #[test]
    fn test_remove_fact() {
        let mut st = SymbolTable::new();
        let new_fact = Fact::new(st.intern("isa"), vec![st.intern("Bob"), st.intern("boy")]);
        let mut kb = KnowledgeBase::new(vec![new_fact.clone()], vec![], st);
        match kb.remove_fact(&new_fact) {
            Ok(_) => {}
            Err(e) => println!("{}", e),
        }

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
}

#[cfg(test)]
mod inference_tests {
    use super::*;

    #[test]
    fn test_assert_and_infer() {
        let mut kb = KnowledgeBase::new(vec![], vec![], SymbolTable::new());
        let new_fact = Fact::new(
            kb.intern_string("isa"),
            vec![kb.intern_string("Bob"), kb.intern_string("boy")],
        );

        match kb.assert(new_fact.clone()) {
            Ok(_) => {}
            Err(e) => println!("{}", e),
        }

        let new_rule = Rule::new(
            vec![
                Fact::new(
                    kb.intern_string("isa"),
                    vec![kb.intern_string("?x"), kb.intern_string("boy")],
                ),
            ],
            Fact::new(kb.intern_string("cool"), vec![kb.intern_string("?x")]),
        );

        match kb.assert(new_rule.clone()) {
            Ok(_) => {}
            Err(e) => println!("{}", e),
        }

        let result_fact = Fact::new(kb.intern_string("cool"), vec![kb.intern_string("Bob")]);
        assert_eq!(kb.contains_fact(&new_fact), true);
        assert_eq!(kb.contains_rule(&new_rule), true);
        assert_eq!(kb.contains_fact(&result_fact), true);
    }

    #[test]
    fn test_infer_rule() {
        let mut kb = KnowledgeBase::new(vec![], vec![], SymbolTable::new());
        let new_fact = Fact::new(
            kb.intern_string("isa"),
            vec![kb.intern_string("Bob"), kb.intern_string("boy")],
        );

        match kb.assert(new_fact.clone()) {
            Ok(_) => {}
            Err(e) => println!("{}", e),
        }

        let new_rule = Rule::new(
            vec![
                Fact::new(
                    kb.intern_string("isa"),
                    vec![kb.intern_string("?x"), kb.intern_string("boy")],
                ),
                Fact::new(
                    kb.intern_string("was"),
                    vec![kb.intern_string("?x"), kb.intern_string("?y")],
                ),
            ],
            Fact::new(kb.intern_string("cool"), vec![kb.intern_string("?y")]),
        );

        match kb.assert(new_rule.clone()) {
            Ok(_) => {}
            Err(e) => println!("{}", e),
        }

        let result_rule = Rule::new(
            vec![
                Fact::new(
                    kb.intern_string("was"),
                    vec![kb.intern_string("Bob"), kb.intern_string("?y")],
                ),
            ],
            Fact::new(kb.intern_string("cool"), vec![kb.intern_string("?y")]),
        );
        assert_eq!(kb.contains_fact(&new_fact), true);
        assert_eq!(kb.contains_rule(&new_rule), true);
        assert_eq!(kb.contains_rule(&result_rule), true);
    }

    #[test]
    fn test_bind() {
        let mut kb = KnowledgeBase::new(vec![], vec![], SymbolTable::new());
        let fact1 = Fact::new(
            kb.intern_string("isa"),
            vec![kb.intern_string("Bob"), kb.intern_string("boy")],
        );

        let fact2 = Fact::new(
            kb.intern_string("isa"),
            vec![kb.intern_string("?x"), kb.intern_string("boy")],
        );

        let bindings = match kb.try_bind(&fact1, &fact2) {
            Ok(lst) => lst,
            Err(e) => {
                println!("{}", e);
                HashMap::new()
            }
        };

        assert!(bindings.contains_key(&kb.intern_string("?x")));

        let new_rule = Rule::new(
            vec![
                Fact::new(
                    kb.intern_string("isa"),
                    vec![kb.intern_string("?x"), kb.intern_string("boy")],
                ),
            ],
            Fact::new(kb.intern_string("cool"), vec![kb.intern_string("?x")]),
        );

        let result_fact = kb.apply_bindings(&new_rule.rhs, &bindings);

        assert_eq!(
            result_fact,
            Fact::new(kb.intern_string("cool"), vec![kb.intern_string("Bob")])
        );
    }

    #[test]
    fn test_has_var() {
        let mut kb = KnowledgeBase::new(vec![], vec![], SymbolTable::new());

        let fact = Fact::new(
            kb.intern_string("isa"),
            vec![kb.intern_string("?x"), kb.intern_string("boy")],
        );

        assert_eq!(true, kb.has_var(&fact));

        let fact = Fact::new(
            kb.intern_string("isa"),
            vec![kb.intern_string("Bob"), kb.intern_string("boy")],
        );

        assert_eq!(false, kb.has_var(&fact));
    }
}

#[cfg(test)]
mod query_tests {
    use super::*;

    #[test]
    fn empty_test() {
        let mut kb = KnowledgeBase::new(vec![], vec![], SymbolTable::new());
        let f = Fact::new(kb.intern_string("isa"), vec![kb.intern_string("?a"), kb.intern_string("?b")]);
        let a = kb.query(&f);

        let b : Vec<QueryBinding> = vec![];
        assert_eq!(a,b);
    }

    #[test]
    fn single_binding_test() {
        let mut kb = KnowledgeBase::new(vec![], vec![], SymbolTable::new());
        let facts = vec![vec!["isa","a","b"],vec!["isa","c","d"],vec!["isa","a","c"],vec!["isa","a","d"],vec!["isa","f","g"]];

        for fact in facts {
            let f = Fact::new(kb.intern_string(fact[0]),vec![kb.intern_string(fact[1]),kb.intern_string(fact[2])]);
            kb.assert(f);
        }

        let f = Fact::new(kb.intern_string("isa"), vec![kb.intern_string("f"), kb.intern_string("?b")]);
        let a = kb.query(&f);
        let b : Vec<QueryBinding> = vec![vec![(kb.intern_string("?b"), kb.intern_string("g"))]];
        assert_eq!(a,b);
    }

    #[test]
    fn multi_binding_test() {
        let mut kb = KnowledgeBase::new(vec![], vec![], SymbolTable::new());
        let facts = vec![vec!["isa","a","b","c"],vec!["isa","c","d","c"],vec!["isa","a","c","c"],vec!["isa","a","d","c"],vec!["isa","f","g","c"]];

        for fact in facts {
            let f = Fact::new(kb.intern_string(fact[0]),vec![kb.intern_string(fact[1]),kb.intern_string(fact[2]),kb.intern_string(fact[3])]);
            kb.assert(f);
        }

        let f = Fact::new(kb.intern_string("isa"), vec![kb.intern_string("?a"), kb.intern_string("?b"), kb.intern_string("c")]);

        assert_eq!(kb.query(&f).len(),5);
    }
}