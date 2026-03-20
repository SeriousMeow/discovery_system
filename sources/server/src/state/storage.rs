use std::collections::HashMap;

type KeyType = String;
type ValueType = String;

pub struct Storage {
    storage: HashMap<KeyType, ValueType>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }

    pub fn touch(&mut self, _key: &KeyType) {
        // Initialize resources for key. Not requered for now
    }

    pub fn insert(&mut self, key: &KeyType, value: ValueType) {
        self.storage.insert(key.clone(), value);
    }

    pub fn get(&self, key: &KeyType) -> Option<&ValueType> {
        self.storage.get(key)
    }
}
