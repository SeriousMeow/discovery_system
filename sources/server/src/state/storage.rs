use std::collections::HashMap;

pub type KeyType = String;
pub type ValueType = String;

pub struct Storage {
    storage: HashMap<KeyType, Vec<ValueType>>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }

    pub fn touch(&mut self, key: KeyType) {
        self.storage.insert(key, Vec::new());
    }

    pub(super) fn insert(&mut self, key: &KeyType, value: ValueType) {
        let Some(item) = self.storage.get_mut(key) else {
            return;
        };

        item.push(value);
    }

    pub fn get(&self, key: &KeyType) -> Option<&Vec<ValueType>> {
        self.storage.get(key)
    }
}
