use crate::ir::TypeInformation;

pub struct Context<'a> {
    type_information: &'a TypeInformation,
}

impl<'a> Context<'a> {
    pub fn new(type_information: &'a TypeInformation) -> Self {
        Self { type_information }
    }

    pub fn type_information(&self) -> &TypeInformation {
        self.type_information
    }
}
