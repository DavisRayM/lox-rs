use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{errors::RuntimeError, token::Literal};

#[derive(Debug, Clone)]
pub struct Environment {
    store: HashMap<String, Literal>,
    // Everything related to this is so janky.......
    enclosing: Option<Arc<Mutex<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn enclosing(&mut self, env: Arc<Mutex<Environment>>) {
        self.enclosing = Some(env);
    }

    pub fn define(&mut self, k: String, v: Literal) -> Result<(), RuntimeError> {
        self.store.insert(k, v);
        Ok(())
    }

    pub fn assign(&mut self, k: String, v: Literal) -> Result<(), RuntimeError> {
        if self.store.get(&k).is_some() {
            self.store.insert(k, v);
            return Ok(());
        }

        if self.enclosing.is_some() {
            let enclosing = self.enclosing.take().unwrap();
            let cloned_env = Arc::clone(&enclosing);
            self.enclosing = Some(enclosing);
            let mut env = cloned_env.lock().unwrap();
            return env.assign(k, v);
        }

        Err(RuntimeError {
            cause: format!("undefined variable '{}'", k),
        })
    }

    pub fn get(&mut self, k: &String) -> Result<Literal, RuntimeError> {
        if let Some(literal) = self.store.get(k) {
            return Ok(literal.to_owned());
        }

        if self.enclosing.is_some() {
            let enclosing = self.enclosing.take().unwrap();
            let cloned_env = Arc::clone(&enclosing);
            self.enclosing = Some(enclosing);
            let mut env = cloned_env.lock().unwrap();
            return env.get(k);
        }

        Err(RuntimeError {
            cause: format!("undefined variable '{}'", k),
        })
    }
}
