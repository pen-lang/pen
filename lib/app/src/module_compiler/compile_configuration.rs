pub struct CompileConfiguration {
    pub heap: HeapConfiguration,
    pub list_type: ListTypeConfiguration,
    pub string_type: StringTypeConfiguration,
    pub error_type: ErrorTypeConfiguration,
}

pub type HeapConfiguration = fmm_llvm::HeapConfiguration;
pub type ListTypeConfiguration = lang::hir_mir::ListTypeConfiguration;
pub type StringTypeConfiguration = lang::hir_mir::StringTypeConfiguration;
pub type ErrorTypeConfiguration = lang::hir_mir::ErrorTypeConfiguration;
