test_valgrind_log() {
  grep 'All heap blocks were freed -- no leaks are possible' $1 ||
    (
      grep 'definitely lost: 0 bytes in 0 blocks' $1 &&
        grep 'indirectly lost: 0 bytes in 0 blocks' $1 &&
        grep '0 errors from 0 contexts' $1
    )
}

install_nightly_component() {
  rustup install nightly
  rustup component add --toolchain nightly $1
}

prepare_integration_test() {
  directory=$1

  export PATH=$directory/target/release:$directory/tools:$PATH
  export RUSTC_WRAPPER=sccache
  export PEN_ROOT=$directory

  cargo install turtle-build
}
