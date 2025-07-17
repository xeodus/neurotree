use std::{any::Any, collections::HashMap};

pub struct BlackBoard {
    pub data: HashMap<String, Box<dyn Any + Send + Sync>>
}

impl BlackBoard {
    pub fn new() -> Self {
        Self {
            data: HashMap::new()
        }
    }

    pub fn get<T: 'static + Send + Sync>(&mut self, key: &str, value: T) {
        self.data.insert(key.to_string(), Box::new(value));
    }

    pub fn contains_key(&self, key: &str) -> bool {
        if !self.data.contains_key(key) {
            return false;
        }
        true
    }

    pub fn remove(&mut self, key: &str) -> bool {
        self.data.get(key).is_some()
    }
}
