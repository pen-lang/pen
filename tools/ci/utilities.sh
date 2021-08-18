set_homebrew_llvm_environment_variables() {
  llvm_prefix=$(brew --prefix)/opt/llvm@12

  echo LLVM_SYS_120_PREFIX=$llvm_prefix >>$GITHUB_ENV
  echo PATH=$llvm_prefix/bin:$PATH >>$GITHUB_ENV
}
