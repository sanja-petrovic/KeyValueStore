use std::{collections::HashMap, hash::Hash};
#[derive(Debug)]
pub enum Value<'a> {
    String(&'a str),
    Int(i64),
    Double(f64),
}

pub struct KeyValueStore<K, Value> {
    data: HashMap<K, Value>,
}

impl<K: Eq + PartialEq + Hash + std::fmt::Debug, Value: std::fmt::Debug> KeyValueStore<K, Value> {
    pub fn new() -> KeyValueStore<K, Value> {
        KeyValueStore {
            data: HashMap::new(),
        }
    }

    pub fn put(&mut self, key: K, value: Value) {
        self.data.insert(key, value);
    }

    pub fn get(&self, key: &K) -> Option<&Value> {
        self.data.get(key)
    }

    pub fn print(&self) {
        for (key, value) in &self.data {
            println!("Key: {:?}, Value: {:?}", key, value);
        }
    }
}
