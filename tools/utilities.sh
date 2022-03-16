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

prepare_unit_test() {
  export RUST_MIN_STACK=8388608
}
