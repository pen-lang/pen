pub struct CompileConfiguration {
    pub heap: HeapConfiguration,
    pub list_type: ListTypeConfiguration,
}

pub type HeapConfiguration = fmm_llvm::HeapConfiguration;
pub type ListTypeConfiguration = lang::hir_mir::ListTypeConfiguration;
