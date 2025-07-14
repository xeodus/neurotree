use std::{any::Any, collections::HashMap};

pub struct BlackBoard {
    pub data: HashMap<String, Box<dyn Any>>
}

impl BlackBoard {
    pub fn new() -> Self {
        Self {
            data: HashMap::new()
        }
    }

    pub fn set<T: 'static>(&mut self, key: &str, value: T) {
        self.data.insert(key.into(), Box::new(value));
    }

    pub fn get<T: 'static>(&mut self, key: &str) -> Option<&T> {
        self.data.get(key).and_then(|f| f.downcast_ref::<T>())
    }
}
