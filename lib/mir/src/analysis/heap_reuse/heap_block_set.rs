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
        self.counts.insert(type_.clone(), self.get(type_) + 1);
    }

    pub fn remove(&mut self, type_: &Type) {
        let count = self.get(type_);

        if count == 1 {
            self.counts.remove(type_);
        } else {
            self.counts.insert(type_.clone(), count - 1);
        }
    }

    pub fn merge(&mut self, blocks: &Self) {
        for (type_, count) in &blocks.counts {
            self.counts.insert(type_.clone(), self.get(type_) + count);
        }
    }

    pub fn difference<'a>(&'a self, other: &'a Self) -> impl Iterator<Item = (&Type, usize)> + 'a {
        self.counts
            .iter()
            .filter_map(|(type_, count)| {
                count
                    .checked_sub(other.get(type_))
                    .map(|count| (type_, count))
            })
            .filter(|(_, count)| *count > 0)
    }

    pub fn max(&mut self, other: &Self) {
        for type_ in other.counts.keys() {
            self.counts
                .insert(type_.clone(), self.get(type_).max(other.get(type_)));
        }
    }
}
