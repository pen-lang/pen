test_valgrind_log() {
  if !(grep 'definitely lost: 0 bytes in 0 blocks' $1 &&
      grep 'indirectly lost: 0 bytes in 0 blocks' $1 &&
      grep '0 errors from 0 contexts' $1); then
    cat $1
    exit 1
  fi
}

install_nightly_component() {
  rustup install nightly
  rustup component add --toolchain nightly $1
}

prepare_unit_test() {
  export RUST_MIN_STACK=8388608
}
