use super::{object::StringObject, value::Val};

#[derive(Clone, Debug)]
pub struct Entry {
    pub key: StringObject,
    pub value: Val,
}

impl Default for Entry {
    fn default() -> Self {
        Self::new(StringObject::new(""), Val::nil())
    }
}

impl Entry {
    pub fn new(key: StringObject, value: Val) -> Self {
        Self { key, value }
    }
}

#[derive(Clone, Debug)]
pub struct Table {
    pub entries: Vec<Entry>,
    pub count: usize,
    pub capacity: usize,
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

impl Table {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            count: 0,
            capacity: 0,
        }
    }

    pub fn set_table(&mut self, key: StringObject, value: Val) -> bool {
        if self.count + 1 > (self.capacity as f64 * 0.75) as usize {
            let capacity = if self.capacity == 0 {
                8
            } else {
                self.capacity * 2
            };

            self.adjust_capacity(capacity);
        }
        let entry = self.find_entry(&key);
        let is_new_key = entry.key.is_empty();
        if is_new_key && entry.value.is_nil() {
            self.count += 1;
        }

        let index = (key.hash as usize) % self.capacity;
        self.entries[index].key = key;
        self.entries[index].value = value;

        is_new_key
    }

    pub fn table_add_all(&mut self, table: &mut Table) {
        for entry in table.entries.iter() {
            self.set_table(entry.key.clone(), entry.value.clone());
        }
    }

    pub fn table_get(&mut self, key: &StringObject) -> Option<Val> {
        if self.count == 0 {
            return None;
        }

        let entry = self.find_entry(key);
        if entry.key.is_empty() {
            return None;
        }

        Some(entry.value.clone())
    }

    pub fn table_delete(&mut self, key: &StringObject) -> bool {
        if self.count == 0 {
            return false;
        }

        let mut index = (key.hash as usize) % self.capacity;
        loop {
            let entry = &mut self.entries[index];
            if entry.key.is_empty() {
                return false;
            }
            if entry.key.hash == key.hash {
                entry.key = StringObject::new("");
                entry.value = Val::nil();
                self.count -= 1;
                return true;
            }
            index = (index + 1) % self.capacity;
        }
    }

    fn adjust_capacity(&mut self, capacity: usize) {
        let mut entries = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            entries.push(Entry::default());
        }

        self.count = 0;

        let old_entries = std::mem::replace(&mut self.entries, Vec::new());
        for entry in old_entries.iter() {
            if entry.key.is_empty() {
                continue;
            }

            let index = (entry.key.hash as usize) % capacity;
            let mut dest = &mut entries[index];
            dest.key = entry.key.clone();
            dest.value = entry.value.clone();
            self.count += 1;
        }

        self.entries = entries;
        self.capacity = capacity;
    }

    fn find_entry(&mut self, key: &StringObject) -> &mut Entry {
        let mut index = (key.hash as usize) % self.capacity;
        let mut tombstone_index: Option<usize> = None;

        loop {
            if self.entries[index].key.is_empty() {
                if self.entries[index].value.is_nil() {
                    return if let Some(idx) = tombstone_index {
                        &mut self.entries[idx]
                    } else {
                        &mut self.entries[index]
                    };
                } else if tombstone_index.is_none() {
                    tombstone_index = Some(index);
                }
            } else if self.entries[index].key == *key {
                return &mut self.entries[index];
            }

            index = (index + 1) % self.capacity;
        }
    }
    pub fn free_table(&mut self) {
        self.entries.clear();
        self.count = 0;
        self.capacity = 0;
    }
}
