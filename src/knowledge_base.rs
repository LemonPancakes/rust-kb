use std::collections::HashMap;

//TODO: Make this actually something before Tuesday

#[derive(Debug)]
pub struct KnowledgeBase {
    fact_trees : HashMap<String,Vec<Option<Box<Fact>>>>
}

#[derive(Debug)]
pub struct Fact {
    pred : String,
    args : Vec<String>
}