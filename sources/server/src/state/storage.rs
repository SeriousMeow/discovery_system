use std::collections::HashMap;

pub type KeyType = String;
pub type ValueType = String;

pub struct Storage {
    queues: HashMap<KeyType, Vec<ValueType>>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            queues: HashMap::new(),
        }
    }

    pub fn ensure_queue(&mut self, key: KeyType) {
        self.queues.entry(key).or_insert_with(Vec::new);
    }

    pub(super) fn insert(&mut self, key: &KeyType, value: ValueType) {
        let Some(q) = self.queues.get_mut(key) else {
            return;
        };
        q.push(value);
    }

    pub fn get(&self, key: &KeyType) -> Option<&Vec<ValueType>> {
        self.queues.get(key)
    }
}
