test_valgrind_log() {
  grep 'definitely lost: 0 bytes in 0 blocks' $1 &&
    grep 'indirectly lost: 0 bytes in 0 blocks' $1 &&
    grep '0 errors from 0 contexts' $1
}

install_nightly_component() {
  rustup install nightly
  rustup component add --toolchain nightly $1
}
