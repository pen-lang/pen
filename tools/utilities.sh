test_valgrind_log() {
  grep 'definitely lost: 0 bytes in 0 blocks' $1
  grep 'indirectly lost: 0 bytes in 0 blocks' $1
  grep '0 errors from 0 contexts' $1
}

check_valgrind_command() {
  if ! which valgrind; then
    echo Valgrind not found! >&2
    "$@"
    exit
  fi
}
