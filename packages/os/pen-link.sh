#!/bin/sh

set -e

convert_library_path_to_flag() {
  basename "$1" | sed 's/^lib\(.*\)\.a$/\1/'
}

print_lib_link_flags() {
  for path in "$@"; do
    echo "println!(\"cargo:rustc-link-lib=$(convert_library_path_to_flag "$path")\");"
    echo "println!(\"cargo:rustc-link-search=$(dirname "$path")\");"
  done
}

while getopts o:t: option; do
  case $option in
  o)
    output=$OPTARG
    ;;
  t)
    target=$OPTARG
    ;;
  esac
done

shift $(expr $OPTIND - 1)

if [ -z "$output" ]; then
  exit 1
elif [ -n "$target" ]; then
  target_option="--target $target"
fi

cd $(dirname $0)

main_archive_path=$1
shift

ffi_directory=ffi/application

cat <<EOF >$ffi_directory/build.rs
fn main() {
  println!("cargo:rustc-link-lib=static=$(convert_library_path_to_flag "$main_archive_path")");
  println!("cargo:rustc-link-search=$(dirname "$main_archive_path")");
  $(print_lib_link_flags "$@")
}
EOF

cd $ffi_directory

cargo build --release --quiet $target_option

binary=target/$target/release/os-app

if [ -r $binary.wasm ]; then
  binary=$binary.wasm
fi

cp $binary $output

if [ $(uname) = Darwin ]; then
  codesign -s - $output
fi
