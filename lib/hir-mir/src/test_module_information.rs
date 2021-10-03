use crate::test_function_information::TestFunctionInformation;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TestModuleInformation {
    path: String,
    functions: BTreeMap<String, TestFunctionInformation>,
}

impl TestModuleInformation {
    pub fn new(
        path: impl Into<String>,
        functions: BTreeMap<String, TestFunctionInformation>,
    ) -> Self {
        Self {
            path: path.into(),
            functions,
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn functions(&self) -> &BTreeMap<String, TestFunctionInformation> {
        &self.functions
    }
}
