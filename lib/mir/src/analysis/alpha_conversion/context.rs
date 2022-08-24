use fnv::FnvHashMap;
use std::cell::RefCell;

#[derive(Debug)]
pub struct Context<'a> {
    name_counts: RefCell<FnvHashMap<&'a str, usize>>,
}

impl<'a> Context<'a> {
    pub fn new() -> Self {
        Self {
            name_counts: Default::default(),
        }
    }

    pub fn rename(&self, name: &'a str) -> String {
        let count = self
            .name_counts
            .borrow()
            .get(name)
            .copied()
            .unwrap_or_default();

        self.name_counts.borrow_mut().insert(&name, count + 1);

        if count == 0 {
            name.into()
        } else {
            format!("{}:{}", name, count)
        }
    }
}
