pub struct CompileConfiguration {
    pub instruction: InstructionConfiguration,
    pub list_type: ListTypeConfiguration,
    pub string_type: StringTypeConfiguration,
    pub error_type: ErrorTypeConfiguration,
}

pub type InstructionConfiguration = fmm_llvm::InstructionConfiguration;
pub type ListTypeConfiguration = hir_mir::ListTypeConfiguration;
pub type StringTypeConfiguration = hir_mir::StringTypeConfiguration;
pub type ErrorTypeConfiguration = hir_mir::ErrorTypeConfiguration;
