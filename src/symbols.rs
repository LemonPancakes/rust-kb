use std::ops::Deref;
use std::rc::{Rc, Weak};
use weak_table::WeakHashSet;

#[derive(Debug, Clone, Hash)]
pub enum Symbol {
    Constant(Rc<str>),
    Variable(Rc<str>),
}

impl Symbol {
    // Differentiates between variables and normal statements
    pub fn is_var(&self) -> bool {
        match self {
            Symbol::Variable(_) => {true},
            Symbol::Constant(_) => {false}
        }
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Symbol) -> bool {
        match self {
            Symbol::Constant(rc_1) => {
                match other {
                    Symbol::Constant(rc_2) => {
                        rc_1.as_ptr() == rc_2.as_ptr()
                    },

                    Symbol::Variable(_) => {
                        false
                    }
                }
            },

            Symbol::Variable(rc_1) => {
                match other {
                    Symbol::Constant(_) => {
                        false
                    },

                    Symbol::Variable(rc_2) => {
                        rc_1.as_ptr() == rc_2.as_ptr()
                    }
                }
            }
        }
    }
}

impl Eq for Symbol {}

impl Deref for Symbol {
    type Target = str;
    fn deref(&self) -> &str {
        match self {
            Symbol::Constant(rc) => {
                &rc
            },

            Symbol::Variable(rc) => {
                &rc
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct SymbolTable(WeakHashSet<Weak<str>>);

impl SymbolTable {
    pub fn new() -> Self {
        Self::default()
    }

    // Returns a reference counted pointer for a given string slice
    // This function assumes the string slice is a properly formatted argument
    pub fn intern(&mut self, name: &str) -> Symbol {
        if let Some(rc) = self.0.get(name) {
            SymbolTable::create_symbol(rc)
        } else {
            let rc = Rc::<str>::from(name);
            self.0.insert(Rc::clone(&rc));
            SymbolTable::create_symbol(rc)
        }
    }

    fn create_symbol(rc : Rc<str>) -> Symbol {
        if rc.len() == 0 {
            return Symbol::Constant(rc);
        }

        if &rc[..1] == "?" {
            Symbol::Variable(rc)
        } else {
            Symbol::Constant(rc)
        }
    }
}

#[test]
fn interning() {
    let mut tab = SymbolTable::new();

    let a0 = tab.intern("a");
    let a1 = tab.intern("a");
    let b = tab.intern("b");

    assert_eq!(a0, a1);
    assert_ne!(a0, b);
}

#[test]
fn variable() {
    let mut tab = SymbolTable::new();

    let a = tab.intern("ab");
    let b = tab.intern("?a");

    assert_eq!(a.is_var(), false);
    assert_eq!(b.is_var(), true)
}
