use crate::types::TypeInformation;

pub struct Context {
    type_information: TypeInformation,
}

impl Context {
    pub fn new(type_information: TypeInformation) -> Self {
        Self { type_information }
    }

    pub fn type_information(&self) -> &TypeInformation {
        &self.type_information
    }
}
