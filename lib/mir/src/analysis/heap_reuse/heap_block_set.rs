use crate::types::Type;
use fnv::FnvHashMap;

#[derive(Clone, Debug, Default)]
pub struct HeapBlockSet {
    counts: FnvHashMap<Type, usize>,
}

impl HeapBlockSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, type_: &Type) -> Option<usize> {
        self.counts.get(type_).copied()
    }

    pub fn merge(&mut self, other: &Self) {
        for (type_, &count) in &other.counts {
            self.update_count(type_, count as isize);
        }
    }

    pub fn difference(&mut self, other: &Self) {
        for (type_, &count) in &other.counts {
            self.update_count(type_, -(count as isize));
        }
    }

    pub fn add(&mut self, type_: &Type) {
        self.update_count(type_, 1);
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
