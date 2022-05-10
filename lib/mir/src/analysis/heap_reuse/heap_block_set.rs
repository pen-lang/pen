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

    pub fn difference(&self, other: &Self) -> Self {
        let mut this = self.clone();

        for (type_, &count) in &other.counts {
            this.update_count(type_, -(count as isize));
        }

        this
    }

    pub fn max(&self, other: &Self) -> Self {
        let mut this = self.clone();

        for type_ in self.counts.keys().chain(other.counts.keys()) {
            this.counts
                .insert(type_.clone(), self.get(type_).max(other.get(type_)));
        }

        this
    }

    pub fn is_empty(&self) -> bool {
        self.counts.values().sum::<usize>() == 0
    }

    fn update_count(&mut self, type_: &Type, count: isize) {
        let original_count = self.get(type_) as isize;

        if original_count <= -count {
            self.counts.remove(type_);
        } else {
            self.counts
                .insert(type_.clone(), (original_count + count) as usize);
        }
    }
}
