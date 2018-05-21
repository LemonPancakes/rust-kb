#[derive(Debug, PartialEq)]
pub struct Fact<'a> {
    pub pred: &'a str,
    pub args: Vec<&'a str>,
}

#[derive(Debug, PartialEq)]
pub struct Rule<'a> {
    pub lhs: Vec<Vec<&'a str>>,
    pub rhs: Vec<&'a str>,
}

#[derive(Debug, PartialEq)]
pub struct KB<'a> {
    pub facts: Vec<Fact<'a>>,
    pub rules: Vec<Rule<'a>>,
}