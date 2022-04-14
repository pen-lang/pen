pub struct CompileConfiguration {
    pub fmm: FmmConfiguration,
    pub mir: MirConfiguration,
    pub hir: HirConfiguration,
}

pub type FmmConfiguration = fmm_llvm::InstructionConfiguration;
pub type MirConfiguration = mir_fmm::Configuration;
pub type HashConfiguration = hir_mir::HashConfiguration;
pub type HirConfiguration = hir_mir::CompileConfiguration;
pub type ConcurrencyConfiguration = hir_mir::ConcurrencyConfiguration;
pub type ListTypeConfiguration = hir_mir::ListTypeConfiguration;
pub type MapTypeConfiguration = hir_mir::MapTypeConfiguration;
pub type MapTypeIterationConfiguration = hir_mir::MapTypeIterationConfiguration;
pub type StringTypeConfiguration = hir_mir::StringTypeConfiguration;
pub type ErrorTypeConfiguration = hir_mir::ErrorTypeConfiguration;
