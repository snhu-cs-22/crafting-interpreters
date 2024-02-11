use super::object::StringObj;
use super::value::Value;

#[inline]
fn grow_capacity(capacity: usize) -> usize {
    if capacity < 8 {
        8
    } else {
       capacity * 2
    }
}

#[derive(Default, Clone, Debug)]
pub struct Entry {
    key: Option<StringObj>,
    value: Value,
}

pub struct Table {
    entries: Vec<Entry>,
}

impl Table {
    const TABLE_MAX_LOAD: f64 = 0.75;

    pub fn new() -> Table {
        Table {
            entries: Vec::with_capacity(8),
        }
    }

    pub fn set(&mut self, key: &StringObj, value: &Value) -> bool {
        if (self.entries.len() + 1) as f64 > self.entries.capacity() as f64 * Self::TABLE_MAX_LOAD {
            self.entries.resize(grow_capacity(self.entries.capacity()), Default::default());
        }

        let entry = self.find_entry(&key);
        let is_new_key = entry.is_none();

        if is_new_key {
            self.entries.resize(self.entries.len() + 1, Default::default());
            let entry = Entry {
                key: Some(key.clone()),
                value: value.clone(),
            };
            self.entries.push(entry);
        }
        is_new_key
    }

    pub fn delete(&mut self, key: &StringObj) -> bool {
        if self.entries.len() == 0 {
            return false;
        }

        // Find the entry.
        if let Some(ref mut entry) = self.find_entry(key) {
            // Place a tombstone in the entry.
            entry.key = None;
            entry.value = Value::Bool(true);
            return true;
        } else {
            return false;
        }
    }

    pub fn add_all(&mut self, to: &mut Table) {
        for entry in &self.entries {
            if let Some(key) = &entry.key {
                to.set(&key, &entry.value);
            }
        }
    }

    fn find_entry(&self, key: &StringObj) -> Option<Entry> {
        let mut index = key.hash as usize % self.entries.capacity();
        let mut tombstone = None;
        loop {
            let entry = self.entries.get(index).unwrap();
            if entry.key == None {
                if entry.value == Value::Nil {
                    // Empty entry.
                    return Some(tombstone.unwrap_or(entry.clone()))
                } else {
                    // We found a tombstone.
                    if tombstone.is_none() {
                        tombstone = Some(entry.clone());
                    }
                }
            } else if entry.key == Some(key.clone()) {
                return Some(entry.clone());
            }

            index = (index + 1) % self.entries.capacity();
        }
    }

    pub fn get(&self, key: &StringObj) -> Option<Value> {
        self.find_entry(key).map(|entry| entry.value)
    }
}

pub fn hash_string(key: &str) -> u32 {
    let key = key.as_bytes();

    let mut hash = 2166136261u32;
    for byte in key {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(16777619);
    }
    hash
}
