pub struct CompileConfiguration {
    pub fmm: FmmConfiguration,
    pub hir: HirConfiguration,
}

pub type FmmConfiguration = fmm_llvm::InstructionConfiguration;
pub type HirConfiguration = hir_mir::CompileConfiguration;
pub type ConcurrencyConfiguration = hir_mir::ConcurrencyConfiguration;
pub type ListTypeConfiguration = hir_mir::ListTypeConfiguration;
pub type StringTypeConfiguration = hir_mir::StringTypeConfiguration;
pub type ErrorTypeConfiguration = hir_mir::ErrorTypeConfiguration;
