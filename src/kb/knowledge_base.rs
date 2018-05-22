use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct KnowledgeBase {
    pub facts: HashMap<String, Vec<String>>, // TODO HashMap<Rc<Predicate>, Vec<Rc<Argument>>>
    pub rules: Vec<Rule>
}

// TODO First parse into this struct, then post process
// data into KnowledgeBase. Or maybe we don't need
// this at all and we can parse directly into KnowledgeBase
#[derive(Debug, PartialEq)]
pub struct ParsedKnowledgeBase {
    pub facts: Vec<Fact>,
    pub rules: Vec<Rule>
}

// TODO Eventually want to use these structs
// (Argument, Predicate) to avoid
// having so many copies of the same data
#[derive(Debug, PartialEq)]
pub struct Argument {
    pub name : String
}

#[derive(Debug, PartialEq)]
pub struct Predicate {
    pub name : String
}

#[derive(Debug, PartialEq)]
pub struct Fact {
    pub pred : String, // TODO Rc<Predicate>
    pub args : Vec<String> // TODO Vec<Rc<Argument>>
}

#[derive(Debug, PartialEq)]
pub struct Rule {
    pub lhs: Vec<Vec<String>>,
    pub rhs: Vec<String>,
}
