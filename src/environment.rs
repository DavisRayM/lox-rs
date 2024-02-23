use std::collections::HashMap;

use crate::token::Literal;

pub struct Environment {
    store: HashMap<String, Literal>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    pub fn define(&mut self, k: String, v: Literal) {
        self.store.insert(k, v);
    }

    pub fn get(&self, k: &String) -> Option<&Literal> {
        self.store.get(k)
    }
}
