use primitive_types::U256;
use std::collections::HashSet;

pub struct AccessList {
    warm_slots: HashSet<U256>,
}

impl Default for AccessList {
    fn default() -> Self {
        Self::new()
    }
}

impl AccessList {
    pub fn new() -> Self {
        AccessList {
            warm_slots: HashSet::new(),
        }
    }

    pub fn add_warm_slot(&mut self, slot: U256) {
        self.warm_slots.insert(slot);
    }

    pub fn is_warm_slot(&self, slot: U256) -> bool {
        self.warm_slots.contains(&slot)
    }
}
