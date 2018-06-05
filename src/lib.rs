#![allow(dead_code)]
//! Rust KB is a library implementing basic knowledge base function in Rust. Rust KB provides an
//! easy interface to create knowledge bases from full knowledge base files, or rule/fact strings
//! individually.
//!
//! # Usage
//!
//! This crate is not yet on [crates.io](https://crates.io). Therefore, to use this library, download
//! the source code and add it as a module.
//!
//! Add this to your crate root:
//!
//! ```rust
//! extern crate rust_kb;
//! ```

extern crate nom;
extern crate weak_table;

mod parser;
mod symbols;

use parser::{parse_fact, parse_kb_from_file, parse_rule, ParsedFact, ParsedKnowledgeBase,
             ParsedRule};
use symbols::{Symbol, SymbolTable};

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// Defines a fact relationship between two or more arguments
///
/// A fact can only be created by an instance of a knowledge base. This means that facts cannot be
/// created on its own, or passed from one knowledge base to another.
///
///  # Example
///
/// ```
/// use rust_kb::KnowledgeBase;
///
/// let mut kb = KnowledgeBase::new();
/// match kb.create_fact("fact: (isa square rectangle);") {
///     Ok(fact) => { /* Here you can use the fact object */ },
///     Err(_) => {},
/// }
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Fact {
    pred: Symbol,
    args: Vec<Symbol>,
    asserted: bool,
    supported_by: Vec<(Rc<Fact>, Rc<Rule>)>,
}

impl Fact {
    fn new(pred: Symbol, args: Vec<Symbol>, supported_by: Vec<(Rc<Fact>, Rc<Rule>)>) -> Fact {
        let asserted = supported_by.is_empty();
        Fact {
            pred,
            args,
            asserted,
            supported_by,
        }
    }

    /// Creates a new fact from the parser output and a given symbol table
    fn from(pf: &ParsedFact, symbols: &mut SymbolTable) -> Fact {
        let pred = symbols.intern(&pf.pred);
        let mut args = Vec::new();
        for parg in &pf.args {
            args.push(symbols.intern(&parg));
        }

        Fact::new(pred, args, vec![])
    }

    /// Creates a fact from a vector of Strings, each representing a token in the fact. A symbol
    /// table must also be provided
    fn from_raw(raw_fact: &[String], symbols: &mut SymbolTable) -> Fact {
        let mut args = Vec::new();
        let mut pred = symbols.intern("");
        for (i, item) in raw_fact.iter().enumerate() {
            if i == 0 {
                pred = symbols.intern(&item);
            } else {
                args.push(symbols.intern(&item));
            }
        }

        Fact::new(pred, args, vec![])
    }

    fn contains_variable(&self) -> bool {
        for s in &self.args {
            if s.is_var() {
                return true;
            }
        }
        false
    }
}

/// Defines a knowledge base fact that can be inferred, given 1 or more facts as the premise
///
/// Allows structures and complex dependencies to be imposed on a knowledge base without the
/// need for every rule to be defined explictly. When a rule is added to a knowledge base,
/// it will use logical inference by forward chaining to automatically create the implied rules.
/// Use this if you want to impose specialized relationships that are not the default assumption
/// for rules.
///
/// A rule can only be created by an instance of a knowledge base. This means that rules cannot be
/// created on its own, or passed from one knowledge base to another.
///
///  # Example
///
/// ```
/// use rust_kb::KnowledgeBase;
///
/// let mut kb = KnowledgeBase::new();
/// match kb.create_rule("rule: ((inst ?x ?y) (isa ?y ?z)) -> (inst ?x ?z);") {
///     Ok(rule) => { /* Here you can use the rule object */ },
///     Err(_) => {},
/// }
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Rule {
    lhs: Vec<Fact>,
    rhs: Fact,
    asserted: bool,
    supported_by: Vec<(Rc<Fact>, Rc<Rule>)>,
}

impl Rule {
    /// Create a new rule from Facts
    pub fn new(lhs: Vec<Fact>, rhs: Fact, supported_by: Vec<(Rc<Fact>, Rc<Rule>)>) -> Rule {
        let asserted = supported_by.is_empty();
        Rule {
            lhs,
            rhs,
            asserted,
            supported_by,
        }
    }

    // Create a new rule from a parsed object
    fn from(pr: &ParsedRule, symbols: &mut SymbolTable) -> Rule {
        let mut lhs = Vec::new();

        for parsed_raw_fact in &pr.lhs {
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
            lhs.push(Fact::new(pred, args, vec![]));
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
        let rhs = Fact::new(pred, args, vec![]);

        Rule::new(lhs, rhs, vec![])
    }
}

/// Abstraction that encompasses facts and rules
///
/// Use this trait when objects can be either facts and rules, and then do case handling depending
/// on the specific statement passed in
pub trait Statement {
    /// Should return Some(Fact) when the Statement is a fact, None otherwise
    fn to_fact(&self) -> Option<Fact>;

    /// Should return Some(Rule) when the Statement is a rule, None otherwise
    fn to_rule(&self) -> Option<Rule>;
}

impl Statement for Fact {
    #[inline]
    fn to_fact(&self) -> Option<Fact> {
        Some(self.clone())
    }

    #[inline]
    fn to_rule(&self) -> Option<Rule> {
        None
    }
}

impl Statement for Rule {
    #[inline]
    fn to_fact(&self) -> Option<Fact> {
        None
    }

    #[inline]
    fn to_rule(&self) -> Option<Rule> {
        Some(self.clone())
    }
}

// Type alias for an index of one argument column
type ArgumentHash = HashMap<Symbol, Vec<Rc<Fact>>>;

// Type alias for a specific binding from one variable to one argument
type QueryBinding = Vec<(Symbol, Symbol)>;

/// A data structure which can take in facts and rules, and respond to logical questions and queries
///
/// A knowledge base can take (as well as remove) facts and rules to generate facts which it
/// knows to be true. Then, the knowledge base can be asked logical questions and will answer whether
/// it can determine the answer.
#[derive(Debug)]
pub struct KnowledgeBase {
    facts: Vec<Rc<Fact>>,
    facts_map: HashMap<Symbol, Vec<ArgumentHash>>,
    rules: Vec<Rc<Rule>>,
    symbols: SymbolTable,
}

impl PartialEq for KnowledgeBase {
    fn eq(&self, other: &KnowledgeBase) -> bool {
        //TODO Make this work as expected. AKA not depend on order of insertion
        self.facts == other.facts && self.rules == other.rules
    }
}

impl KnowledgeBase {
    /// Creates a new, empty knowledge base
    ///
    /// This function should be used when one is programmatically building up a knowledge base and
    /// no knowledge is known before the start of the program. If any static facts or rules are
    /// known beforehand, a knowledge base file should be used, and KnowledgeBase::from_file() should
    /// create the knowledge base.
    ///
    ///  # Example
    ///
    /// ```
    /// use rust_kb::KnowledgeBase;
    ///
    /// let mut kb = KnowledgeBase::new();
    /// // kb knows nothing and all asks will return false
    /// ```
    pub fn new() -> KnowledgeBase {
        KnowledgeBase {
            facts: Vec::new(),
            facts_map: HashMap::new(),
            rules: Vec::new(),
            symbols: SymbolTable::new(),
        }
    }

    // Creates a new knowledge base with given rules and facts and a symbol table
    fn new_filled(facts: Vec<Fact>, rules: Vec<Rule>, symbols: SymbolTable) -> KnowledgeBase {
        let mut kb = KnowledgeBase {
            facts: Vec::new(),
            facts_map: HashMap::new(),
            rules: Vec::new(),
            symbols,
        };

        for fact in facts {
            kb.assert(fact);
        }

        for rule in rules {
            kb.assert(rule);
        }

        kb
    }

    /// Creates a knowledge base from a parsed object from the crate's parser
    fn from(pkb: ParsedKnowledgeBase) -> KnowledgeBase {
        let mut facts = Vec::new();
        let mut rules = Vec::new();
        let mut symbols = SymbolTable::new();

        for parsed_fact in &pkb.facts {
            let f = Fact::from(&parsed_fact, &mut symbols);
            if !f.contains_variable() {
                facts.push(f);
            }
        }

        for parsed_rule in &pkb.rules {
            rules.push(Rule::from(&parsed_rule, &mut symbols));
        }

        KnowledgeBase::new_filled(facts, rules, symbols)
    }

    /// Attempts to create a knowledge base from a given input file
    ///
    /// The input file should formatted as follows. The entire file should be surrounded by kb { ... }
    ///
    ///  # Proper knowledge base delimiters
    ///
    /// ``` txt
    /// kb {
    ///     ...
    /// }
    /// ```
    ///
    /// All facts should follow. Each fact should be on its own line and be prefixed by "fact:".
    /// Then, the fact should be left left parenthesis, the predicate, one or more arguments, and finally
    /// a right parenthesis
    ///
    /// # Proper fact syntax
    ///
    /// ``` txt
    /// kb {
    ///     fact: (isa cube box)
    ///     fact: (isa box container)
    ///
    ///     ...
    /// }
    /// ```
    ///
    /// Finally, all rules should follow. Each rule should be on its own line and be prefixed by "rule:".
    /// Then the rule should have a list of one or more facts, a right arrow (->), and finally a fact
    /// that can be inferred. These facts should use variables to connect arguments from different
    /// facts.
    ///
    /// # Proper knowledge base file format
    ///
    /// ``` txt
    /// kb {
    ///     fact: (isa cube box)
    ///     fact: (isa box container)
    ///
    ///     rule: ((inst ?x ?y) (isa ?y ?z)) -> (inst ?x ?z)
    /// }
    /// ```
    ///
    ///  # Example
    ///
    /// ``` ignore,
    /// use rust_kb::KnowledgeBase;
    ///
    /// let mut kb = KnowledgeBase::from_file("initial_state.kb");
    ///
    /// // The knowledge base now has all facts and rules from the file
    /// ```
    pub fn from_file(filename: &str) -> Result<KnowledgeBase, String> {
        let pkb = parse_kb_from_file(filename)?;
        Ok(KnowledgeBase::from(pkb))
    }

    /// Attempts to create a fact from a given string slice.
    ///
    /// If the fact is ill-formatted, the function will return an error. In this context, the
    /// fact must be terminated by a semicolon.
    ///
    ///  # Example
    ///
    /// ```
    /// use rust_kb::KnowledgeBase;
    ///
    /// let mut kb = KnowledgeBase::new();
    /// match kb.create_fact("fact: (isa square rectangle);") {
    ///     Ok(fact) => { /* Will execute this branch */ },
    ///     Err(_) => { /* Will not execute this branch, because of proper format */ },
    /// }
    /// ```
    pub fn create_fact(&mut self, fact: &str) -> Result<Fact, String> {
        let pf = parse_fact(fact.as_bytes())?;
        Ok(Fact::from(&pf, &mut self.symbols))
    }

    /// Attempts to create a rule from a given string slice.
    ///
    /// If the rule is ill-formatted, the function will return an error. In this context, the
    /// fact must be terminated by a semicolon.
    ///
    ///  # Example
    ///
    /// ```
    /// use rust_kb::KnowledgeBase;
    ///
    /// let mut kb = KnowledgeBase::new();
    /// match kb.create_rule("rule: ((inst ?x ?y) (isa ?y ?z)) -> (inst ?x ?z);") {
    ///     Ok(fact) => { /* Will execute this branch */ },
    ///     Err(_) => { /* Will not execute this branch, because of proper format */ },
    /// }
    /// ```
    pub fn create_rule(&mut self, rule: &str) -> Result<Rule, String> {
        let pr = parse_rule(rule.as_bytes())?;
        Ok(Rule::from(&pr, &mut self.symbols))
    }

    #[inline]
    fn intern_string(&mut self, name: &str) -> Symbol {
        self.symbols.intern(name)
    }

    /// Add a fact or rule to the knowledge base
    ///
    /// This function will use inference by forward chaining to add implied facts from given rules.
    /// An error will be returned if the statement is already present in the knowledge base. Use this
    /// error to detect logical errors, or duplicate assertions in code.
    ///
    ///  # Example
    ///
    /// ```
    /// use rust_kb::KnowledgeBase;
    ///
    /// let mut kb = KnowledgeBase::new();
    /// match kb.create_fact("fact: (isa square rectangle);") {
    ///     Ok(fact) => { kb.assert(fact); },
    ///     Err(_) => {},
    /// }
    /// ```
    pub fn assert<T: Statement>(&mut self, statement: T) -> Result<Rc<Statement>, String> {
        match statement.to_fact() {
            //TODO: Check if fact has bound variables
            Some(fact) => match self.add_fact(fact) {
                Ok(rc_fact) => {
                    for rule in &self.rules.clone() {
                        self.infer(rc_fact.clone(), rule.clone());
                    }
                    Ok(rc_fact)
                }
                Err(e) => Err(e),
            },
            None => {
                let rule = statement.to_rule().unwrap();
                match self.add_rule(rule) {
                    Ok(rc_rule) => {
                        for fact in &self.facts.clone() {
                            self.infer(fact.clone(), rc_rule.clone());
                        }
                        Ok(rc_rule)
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }

    /// Remove a fact or rule from the knowledge base.
    ///
    /// This function will remove a specific statement from the knowledge base. In addition, it will
    /// recursively chain logic to remove other statements that were dependent on the given statement
    ///
    /// Statements that still have support from other Fact/Rule pairs will error on retract
    ///
    ///  # Example
    ///
    /// ```
    /// use rust_kb::{KnowledgeBase,Fact};
    ///
    /// let mut kb = KnowledgeBase::new();
    ///
    /// // Add to the knowledge base
    ///
    /// if let Ok(fact) = kb.create_fact("fact: (isa square rectangle);") {
    ///     kb.retract(fact);
    /// }
    /// ```
    pub fn retract<T: Statement>(&mut self, statement: T) -> Result<(), String> {
        match statement.to_fact() {
            Some(fact) => {
                if fact.supported_by.is_empty() {
                    return self.remove_fact(&fact);
                } else {
                    return Err(String::from("Fact cannot be removed because it's supported"));
                }
            },
            None => {
                let rule = statement.to_rule().unwrap();
                if rule.supported_by.is_empty() {
                    return self.remove_rule(&rule);
                } else {
                    return Err(String::from("Rule cannot be removed because it's supported"));
                }
            }
        }
    }

    /// Ask if a specific fact can be proven by the knowledge base
    ///
    ///  # Example
    ///
    /// ```
    /// use rust_kb::KnowledgeBase;
    ///
    /// let mut kb = KnowledgeBase::new();
    /// if let Ok(fact) = kb.create_fact("fact: (isa square rectangle);") {
    ///     kb.ask(&fact);
    /// }
    /// ```
    pub fn ask(&self, fact: &Fact) -> Result<bool, String> {
        if self.contains_fact(fact) {
            return Ok(true);
        }
        Ok(false)
    }

    // internal method to add a fact to the knowledge base
    // used within the forward chaining algorithm
    fn insert_fact(&mut self, fact: Fact) -> Rc<Fact> {
        let fact_ref = Rc::new(fact);
        self.facts.push(fact_ref.clone());

        let args_vec = self.facts_map
            .entry(fact_ref.pred.clone())
            .or_insert(Vec::new());

        if args_vec.is_empty() {
            for _ in 0..fact_ref.args.len() {
                args_vec.push(HashMap::new());
            }
        }

        for j in 0..args_vec.len() {
            let mut arg_list = args_vec[j]
                .entry(fact_ref.args[j].clone())
                .or_insert(Vec::new());
            arg_list.push(fact_ref.clone());
        }

        fact_ref
    }

    // checks whether fact already exists in knowledge base, and calls internal insert function
    fn add_fact(&mut self, fact: Fact) -> Result<Rc<Fact>, String> {
        if fact.contains_variable() {
            return Err(String::from("Cannot assert fact with bound variables"));
        }

        if self.contains_fact(&fact) {
            return Err(String::from("fact already in kb"));
        }

        Ok(self.insert_fact(fact))
    }

    // attempts to find and remove a fact
    // returns an error if the fact cannot be found
    fn remove_fact(&mut self, fact: &Fact) -> Result<(), String> {
        if fact.contains_variable() {
            return Err(String::from("Cannot retract fact with bound variables"));
        }

        let mut fact_to_remove = None;

        for i in 0..self.facts.len() {
            if fact == &*self.facts[i] {
                fact_to_remove = Some(self.facts[i].clone());
                self.facts.remove(i);
                break;
            }
        }

        match fact_to_remove {
            None => Err(String::from("fact does not exist in kb")),

            Some(fact_reference) => {
                {
                    // A found fact must be in args vec
                    let mut args_vec = self.facts_map.get_mut(&fact_reference.pred).unwrap();

                    for j in 0..args_vec.len() {
                        // A found fact must have an entry for each argument
                        let mut arg_list = args_vec[j].get_mut(&fact_reference.args[j]).unwrap();

                        let index = arg_list.iter().position(|x| *x == fact_reference).unwrap();
                        arg_list.remove(index);
                    }
                }

                // retract facts supported by this fact
                for f in &self.facts.clone() {
                    println!("fact being supported: {:?}\n\n", f);
                    for i in 0..f.supported_by.len() {
                        if fact_reference == f.supported_by[i].0 {
                            self.remove_fact(&f).is_ok();
                        }
                    }
                }

                // retract rules supported by this fact
                for r in self.rules.clone().iter() {
                    for i in 0..r.supported_by.len() {
                        if fact_reference == r.supported_by[i].0 {
                            self.remove_rule(&r).is_ok();
                        }
                    }
                }

                Ok(())
            }
        }
    }

    fn insert_rule(&mut self, rule: Rule) -> Rc<Rule> {
        let rule_ref = Rc::new(rule);
        self.rules.push(rule_ref.clone());

        rule_ref
    }

    // checks whether rule already exists in knowledge base, and calls internal insert function
    fn add_rule(&mut self, rule: Rule) -> Result<Rc<Rule>, String> {
        if self.contains_rule(&rule) {
            return Err(String::from("rule already in kb"));
        }
        Ok(self.insert_rule(rule))
    }

    // attempts to find and remove a rule
    // returns an error if the rule cannot be found
    fn remove_rule(&mut self, rule: &Rule) -> Result<(), String> {
        let mut rule_to_remove = None;

        for i in 0..self.rules.len() {
            if rule == &*self.rules[i] {
                rule_to_remove = Some(self.rules[i].clone());
                self.rules.remove(i);
                break;
            }
        }

        match rule_to_remove {
            None => Err(String::from("rule does not exist in kb")),

            Some(rule_reference) => {
                // retract facts supported by this rule
                for f in &self.facts.clone() {
                    for i in 0..f.supported_by.len() {
                        if rule_reference == f.supported_by[i].1 {
                            Rc::make_mut(&mut f.clone()).supported_by.remove(i);
                            self.remove_fact(&f).is_ok();
                        }
                    }
                }

                // retract rules supported by this rule
                for r in &self.rules.clone() {
                    for i in 0..r.supported_by.len() {
                        if rule_reference == r.supported_by[i].1 {
                            Rc::make_mut(&mut r.clone()).supported_by.remove(i);
                            self.remove_rule(&r).is_ok();
                        }
                    }
                }

                Ok(())
            }
        }
    }

    // checks if given fact is in knowledge base
    fn contains_fact(&self, fact: &Fact) -> bool {
        self.facts.iter().fold(false, |acc, f| {
            let temp = &**f;
            acc || (temp.pred == fact.pred && temp.args == fact.args)
        })
    }

    // checks if given rule is in knowledge base
    fn contains_rule(&self, rule: &Rule) -> bool {
        self.rules.iter().fold(false, |acc, r| {
            let temp = &**r;
            acc || (temp.lhs == rule.lhs && temp.rhs == rule.rhs)
        })
    }

    // function that implements inference by forward chaining
    fn infer(&mut self, fact: Rc<Fact>, rule: Rc<Rule>) {
        // Inference by Forward Chaining
        if rule.lhs.len() == 1 {
            let lhs = &rule.lhs[0];
            if let Ok(bindings) = self.try_bind(&fact, lhs) {
                let new_fact =
                    self.apply_bindings(&rule.rhs, Some((fact.clone(), rule.clone())), &bindings);
                if !self.has_var(&new_fact) {
                    self.assert(new_fact).is_ok();
                }
            }
        } else if rule.lhs.len() > 1 {
            let lhs = &rule.lhs[0];
            if let Ok(bindings) = self.try_bind(&fact, lhs) {
                let new_lhs = rule.lhs
                    .clone()
                    .iter()
                    .enumerate()
                    .filter(|&(n, _)| n != 0)
                    .map(|(_, f)| self.apply_bindings(f, None, &bindings))
                    .collect::<Vec<Fact>>();
                let new_rhs = self.apply_bindings(&rule.rhs, None, &bindings);
                let new_rule = Rule::new(new_lhs, new_rhs, vec![(fact.clone(), rule.clone())]);

                self.assert(new_rule).is_ok();
            }
        }
    }

    fn try_bind(&self, f1: &Fact, f2: &Fact) -> Result<HashMap<Symbol, Symbol>, String> {
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

    fn apply_bindings(
        &self,
        fact: &Fact,
        support: Option<(Rc<Fact>, Rc<Rule>)>,
        bindings: &HashMap<Symbol, Symbol>,
    ) -> Fact {
        let mut args: Vec<Symbol> = Vec::new();
        for symbol in &fact.args {
            if symbol.is_var() {
                if !bindings.contains_key(symbol) {
                    args.push(symbol.clone());
                } else {
                    args.push(bindings.get(symbol).unwrap().clone());
                }
            } else {
                args.push(symbol.clone());
            }
        }
        match support {
            Some(sup) => Fact::new(fact.pred.clone(), args, vec![sup]),
            None => Fact::new(fact.pred.clone(), args, vec![]),
        }
    }

    fn has_var(&self, fact: &Fact) -> bool {
        for symbol in &fact.args {
            if symbol.is_var() {
                return true;
            }
        }
        false
    }

    /// Query a knowledge base to find all possible bindings to variables in the fact
    ///
    /// The given fact should contain at least one variable.
    ///
    /// # Example
    ///
    /// ```
    /// use rust_kb::KnowledgeBase;
    ///
    /// let mut kb = KnowledgeBase::new();
    ///
    /// // Fill the knowledge base
    ///
    /// if let Ok(fact) = kb.create_fact("fact: (isa ?object rectangle);") {
    ///     let result = kb.query(&fact);
    ///     // result is now a vector of all bindings found in the knowledge base.
    /// }
    /// ```
    pub fn query(&self, f: &Fact) -> Vec<QueryBinding> {
        let mut query_indices = vec![];

        for i in 0..f.args.len() {
            if f.args[i].is_var() {
                query_indices.push(i);
            }
        }

        let mut bindings = vec![];

        for matching_fact in self.get_query_facts(f) {
            let mut curr_binding = vec![];
            for i in &query_indices {
                curr_binding.push((f.args[*i].clone(), matching_fact.args[*i].clone()));
            }
            bindings.push(curr_binding);
        }

        bindings
    }

    // returns all of the facts that match the query bindings of the given fact
    fn get_query_facts(&self, f: &Fact) -> Vec<Rc<Fact>> {
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
                                    facts = facts
                                        .intersection(&temp_facts)
                                        .map(|f| f.clone())
                                        .collect();
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
        let mut kb = KnowledgeBase::new();

        if let Ok(new_fact) = kb.create_fact("fact: (isa Bob boy);") {
            match kb.add_fact(new_fact.clone()) {
                Ok(_) => {}
                Err(e) => println!("{}", e),
            }

            assert_eq!(kb.contains_fact(&new_fact), true);
        }
    }

    #[test]
    fn test_remove_fact() {
        let mut kb = KnowledgeBase::new();

        if let Ok(new_fact) = kb.create_fact("fact: (isa Bob boy);") {
            match kb.add_fact(new_fact.clone()) {
                Ok(_) => {}
                Err(e) => println!("{}", e),
            }

            match kb.remove_fact(&new_fact) {
                Ok(_) => {}
                Err(e) => println!("{}", e),
            }

            assert_eq!(kb.contains_fact(&new_fact), false);
            assert_eq!(kb.facts.is_empty(), true);
        }
    }

    #[test]
    fn test_ask_fact_already_in_kb() {
        let mut kb = KnowledgeBase::new();
        if let Ok(new_fact) = kb.create_fact("fact: (isa Bob boy);") {
            match kb.add_fact(new_fact.clone()) {
                Ok(_) => {}
                Err(e) => println!("{}", e),
            }
            assert_eq!(kb.ask(&new_fact), Ok(true));
        }
    }

    #[test]
    fn test_ask_fact_not_in_fb() {
        let mut kb = KnowledgeBase::new();
        if let Ok(new_fact) = kb.create_fact("fact: (isa Bob boy);") {
            assert_eq!(kb.ask(&new_fact), Ok(false));
        }
    }
}

#[cfg(test)]
mod inference_tests {
    use super::*;

    #[test]
    fn test_assert_and_infer() {
        let mut kb = KnowledgeBase::new();
        if let Ok(new_fact) = kb.create_fact("fact: (isa Bob boy);") {
            match kb.assert(new_fact.clone()) {
                Ok(_) => {}
                Err(e) => println!("{}", e),
            }

            if let Ok(new_rule) = kb.create_rule("rule: ((isa ?x boy)) -> (cool ?x);") {
                match kb.assert(new_rule.clone()) {
                    Ok(_) => {}
                    Err(e) => println!("{}", e),
                }

                if let Ok(result_fact) = kb.create_fact("fact: (cool Bob);") {
                    assert_eq!(kb.contains_fact(&new_fact), true);
                    assert_eq!(kb.contains_rule(&new_rule), true);
                    assert_eq!(kb.contains_fact(&result_fact), true);
                }
            }
        }
    }

    #[test]
    fn test_infer_rule() {
        let mut kb = KnowledgeBase::new();
        if let Ok(new_fact) = kb.create_fact("fact: (isa Bob boy);") {
            match kb.assert(new_fact.clone()) {
                Ok(_) => {}
                Err(e) => println!("{}", e),
            }

            if let Ok(new_rule) = kb.create_rule("rule: ((isa ?x boy) (was ?x ?y)) -> (cool ?y);") {
                match kb.assert(new_rule.clone()) {
                    Ok(_) => {}
                    Err(e) => println!("{}", e),
                }

                if let Ok(result_rule) = kb.create_rule("rule: ((was Bob ?y)) -> (cool ?y);") {
                    assert_eq!(kb.contains_fact(&new_fact), true);
                    assert_eq!(kb.contains_rule(&new_rule), true);
                    assert_eq!(kb.contains_rule(&result_rule), true);
                }
            }
        }
    }

    #[test]
    fn test_bind() {
        let mut kb = KnowledgeBase::new();
        if let Ok(fact1) = kb.create_fact("fact: (isa Bob boy);") {
            if let Ok(fact2) = kb.create_fact("fact: (isa ?x boy);") {
                let bindings = match kb.try_bind(&fact1, &fact2) {
                    Ok(lst) => lst,
                    Err(e) => {
                        println!("{}", e);
                        HashMap::new()
                    }
                };

                assert!(bindings.contains_key(&kb.intern_string("?x")));

                if let Ok(new_rule) = kb.create_rule("rule: ((isa ?x boy)) -> (cool ?x);") {
                    let result_fact = kb.apply_bindings(&new_rule.rhs, None, &bindings);

                    assert_eq!(
                        result_fact,
                        Fact::new(
                            kb.intern_string("cool"),
                            vec![kb.intern_string("Bob")],
                            vec![]
                        )
                    );
                }
            }
        }
    }

    #[test]
    fn test_has_var() {
        let mut kb = KnowledgeBase::new();

        if let Ok(fact) = kb.create_fact("fact: (isa ?x boy);") {
            assert_eq!(true, kb.has_var(&fact));

            if let Ok(fact2) = kb.create_fact("fact: (isa Bob boy);") {
                assert_eq!(false, kb.has_var(&fact2));
            }
        }
    }

    #[test]
    fn test_retract_inferred() {
        let mut kb = KnowledgeBase::new();

        if let Ok(new_fact) = kb.create_fact("fact: (isa Bob boy);") {
            let rc_fact = match kb.assert(new_fact.clone()) {
                Ok(_f) => Some(_f),
                Err(e) => {
                    println!("{}", e);
                    None
                }
            };

            if let Ok(new_rule) = kb.create_rule("rule: ((isa ?x boy)) -> (cool ?x);") {
                let rc_rule = match kb.assert(new_rule.clone()) {
                    Ok(_r) => Some(_r),
                    Err(e) => {
                        println!("{}", e);
                        None
                    }
                };

                let result_fact = Fact::new(
                    kb.intern_string("cool"),
                    vec![kb.intern_string("Bob")],
                    vec![
                        (
                            Rc::new(rc_fact.unwrap().to_fact().unwrap()),
                            Rc::new(rc_rule.unwrap().to_rule().unwrap()),
                        ),
                    ],
                );
                assert_eq!(kb.contains_fact(&new_fact), true);
                assert_eq!(kb.contains_rule(&new_rule), true);
                assert_eq!(kb.contains_fact(&result_fact), true);

                for f in kb.facts.iter() {
                    println!("{:?}\n\n", f)
                }

                assert!(kb.retract(new_fact.clone()).is_ok());

                assert_eq!(kb.contains_fact(&new_fact), false);
                assert_eq!(kb.contains_rule(&new_rule), true);
                assert_eq!(kb.contains_fact(&result_fact), false);
            }
        }
    }
}

#[cfg(test)]
mod query_tests {
    use super::*;

    #[test]
    fn empty_test() {
        let mut kb = KnowledgeBase::new();
        if let Ok(f) = kb.create_fact("fact: (isa ?a ?b);") {
            let a = kb.query(&f);
            let b: Vec<QueryBinding> = vec![];

            assert_eq!(a, b);
        }
    }

    #[test]
    fn single_binding_test() {
        let mut kb = KnowledgeBase::new();

        if let Ok(f1) = kb.create_fact("fact: (isa a b);") {
            if let Ok(f2) = kb.create_fact("fact: (isa c d);") {
                if let Ok(f3) = kb.create_fact("fact: (isa a c);") {
                    if let Ok(f4) = kb.create_fact("fact: (isa a d);") {
                        if let Ok(f5) = kb.create_fact("fact: (isa f g);") {
                            let facts = vec![f1, f2, f3, f4, f5];

                            for fact in facts.iter() {
                                match kb.assert(fact.clone()) {
                                    Ok(_) => {}
                                    Err(e) => println!("{}", e),
                                }
                            }

                            if let Ok(f) = kb.create_fact("fact: (isa f ?b);") {
                                let a = kb.query(&f);
                                let b: Vec<QueryBinding> =
                                    vec![vec![(kb.intern_string("?b"), kb.intern_string("g"))]];

                                assert_eq!(a, b);
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn multi_binding_test() {
        let mut kb = KnowledgeBase::new();
        if let Ok(f1) = kb.create_fact("fact: (isa a b c);") {
            if let Ok(f2) = kb.create_fact("fact: (isa c d c);") {
                if let Ok(f3) = kb.create_fact("fact: (isa a c c);") {
                    if let Ok(f4) = kb.create_fact("fact: (isa a d c);") {
                        if let Ok(f5) = kb.create_fact("fact: (isa f g c);") {
                            let facts = vec![f1, f2, f3, f4, f5];

                            for fact in facts.iter() {
                                match kb.assert(fact.clone()) {
                                    Ok(_) => {}
                                    Err(e) => println!("{}", e),
                                }
                            }

                            if let Ok(f) = kb.create_fact("fact: (isa ?a ?b c);") {
                                assert_eq!(kb.query(&f).len(), 5);
                            }
                        }
                    }
                }
            }
        }
    }
}
