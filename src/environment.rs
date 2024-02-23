use std::collections::HashMap;

use crate::{errors::RuntimeError, token::Literal};

pub struct Environment {
    store: HashMap<String, Literal>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    pub fn define(&mut self, k: String, v: Literal) -> Result<(), RuntimeError> {
        self.store.insert(k, v);
        Ok(())
    }

    #[allow(clippy::map_entry)]
    pub fn assign(&mut self, k: String, v: Literal) -> Result<(), RuntimeError> {
        if self.store.contains_key(&k) {
            self.store.insert(k, v);
            Ok(())
        } else {
            Err(RuntimeError {
                cause: format!("undefined variable '{}'", k),
            })
        }
    }

    pub fn get(&self, k: &String) -> Result<Option<&Literal>, RuntimeError> {
        Ok(self.store.get(k))
    }
}
