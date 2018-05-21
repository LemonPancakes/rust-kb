use std::collections::HashMap;

//TODO: Make this actually something before Tuesday

#[derive(Debug, PartialEq)]
pub struct KnowledgeBase {
    pub facts: HashMap<String, Vec<Fact>>,
    pub rules: HashMap<String, Vec<Rule>>
}

// temporary
#[derive(Debug, PartialEq)]
pub struct KB {
    pub facts: Vec<Fact>,
    pub rules: Vec<Rule>
}

#[derive(Debug, PartialEq)]
pub struct Fact {
    pub pred : String,
    pub args : Vec<String>
}

#[derive(Debug, PartialEq)]
pub struct Rule {
    pub lhs: Vec<Vec<String>>,
    pub rhs: Vec<String>,
}
