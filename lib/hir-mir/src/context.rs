use crate::{CompileConfiguration, CompileError};
use fnv::FnvHashMap;
use hir::{
    analysis::{type_collector, AnalysisContext},
    ir::Module,
    types::{self, Type},
};

#[derive(Debug)]
pub struct CompileContext {
    analysis_context: AnalysisContext,
    configuration: Option<CompileConfiguration>,
}

impl CompileContext {
    pub fn new(module: &Module, configuration: Option<CompileConfiguration>) -> Self {
        Self {
            analysis_context: AnalysisContext::new(
                type_collector::collect(module),
                type_collector::collect_records(module),
                configuration.as_ref().map(|configuration| {
                    types::Reference::new(
                        &configuration.error_type.error_type_name,
                        module.position().clone(),
                    )
                    .into()
                }),
            ),
            configuration,
        }
    }

    #[cfg(test)]
    pub fn dummy(
        types: FnvHashMap<String, Type>,
        records: FnvHashMap<String, Vec<types::RecordField>>,
    ) -> Self {
        use super::compile_configuration::COMPILE_CONFIGURATION;
        use position::{test::PositionFake, Position};

        Self {
            analysis_context: AnalysisContext::new(
                types,
                records,
                Some(
                    types::Reference::new(
                        &COMPILE_CONFIGURATION.error_type.error_type_name,
                        Position::fake(),
                    )
                    .into(),
                ),
            ),
            configuration: COMPILE_CONFIGURATION.clone().into(),
        }
    }

    pub fn types(&self) -> &FnvHashMap<String, Type> {
        self.analysis_context.types()
    }

    pub fn records(&self) -> &FnvHashMap<String, Vec<types::RecordField>> {
        self.analysis_context.records()
    }

    pub fn analysis(&self) -> &AnalysisContext {
        &self.analysis_context
    }

    pub fn configuration(&self) -> Result<&CompileConfiguration, CompileError> {
        self.configuration
            .as_ref()
            .ok_or(CompileError::CompileConfigurationNotProvided)
    }
}
