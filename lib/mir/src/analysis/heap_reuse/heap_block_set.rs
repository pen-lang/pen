use crate::types::Type;
use fnv::FnvHashMap;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HeapBlockSet {
    counts: FnvHashMap<Type, usize>,
}

impl HeapBlockSet {
    pub fn get(&self, type_: &Type) -> usize {
        self.counts.get(type_).copied().unwrap_or_default()
    }

    pub fn add(&mut self, type_: &Type) {
        self.update_count(type_, 1);
    }

    pub fn remove(&mut self, type_: &Type) -> bool {
        if self.get(type_) > 0 {
            self.update_count(type_, -1);

            true
        } else {
            false
        }
    }

    fn update_count(&mut self, type_: &Type, count: isize) {
        let count =
            (self.counts.get(&type_).copied().unwrap_or_default() as isize + count) as usize;

        if count == 0 {
            self.counts.remove(type_);
        } else {
            self.counts.insert(type_.clone(), count);
        }
    }
}
