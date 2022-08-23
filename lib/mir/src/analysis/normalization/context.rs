use std::cell::Cell;

#[derive(Debug)]
pub struct Context {
    name_index: Cell<usize>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            name_index: Cell::new(0),
        }
    }

    pub fn generate_name(&self) -> String {
        let index = self.name_index.get();
        let name = format!("anf:v:{}", index);

        self.name_index.set(index + 1);

        name
    }
}
