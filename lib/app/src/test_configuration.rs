pub struct TestConfiguration {
    pub test_package_name: String,
    pub test_module_configuration: TestModuleConfiguration,
}

pub type TestModuleConfiguration = hir_mir::TestModuleConfiguration;
