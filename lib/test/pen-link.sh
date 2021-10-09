#!/bin/sh

set -e

if ! which jq >/dev/null 2>&1; then
  echo jq command not found >&2
  exit 1
fi

convert_library_path_to_flag() {
  basename "$1" | sed 's/^lib\(.*\)\.a$/\1/'
}

print_lib_link_flags() {
  for path in "$@"; do
    echo "println!(\"cargo:rustc-link-lib=$(convert_library_path_to_flag "$path")\");"
    echo "println!(\"cargo:rustc-link-search=$(dirname "$path")\");"
  done
}

while getopts i:o: option; do
  case $option in
  o)
    output=$OPTARG
    ;;
  i)
    test_information=$OPTARG
    ;;
  esac
done

shift $(expr $OPTIND - 1)

if [ -z "$output" -o -z "$test_information" ]; then
  exit 1
fi

cd $(dirname $0)

main_archive_path=$1
shift

cat <<EOF >ffi/build.rs
fn main() {
  println!("cargo:rustc-link-lib=static=$(convert_library_path_to_flag "$main_archive_path")");
  println!("cargo:rustc-link-search=$(dirname "$main_archive_path")");
  $(print_lib_link_flags "$@")
}
EOF

cat <<EOF >ffi/src/main.rs
mod heap;
mod test_result;
mod unreachable;

use test_result::TestResult;

extern "C" {
    fn _pen_test_convert_result(result: ffi::Any) -> ffi::Arc<TestResult>;
}

fn main() {
    #[allow(unused_mut)]
    let mut success: usize = 0;
    #[allow(unused_mut)]
    let mut error: usize = 0;

$(
  for module in $(jq -r '.modules | keys[]' $test_information); do
    echo "println!(\"$module\");"

    functions() {
      jq -r ".modules[\"$module\"].functions" $test_information
    }

    for i in $(functions | jq -r 'keys[]'); do
      name=$(functions | jq -r ".[$i].name")
      foreign_name=$(functions | jq -r ".[$i].foreign_name")

      echo "
        #[link(name = \"main_test\")]
        extern \"C\" { fn $foreign_name() -> ffi::Any; }

        let result: Result<_, _> = unsafe { _pen_test_convert_result($foreign_name()) }.into_result();
        println!(\"\t{}\t$name\", if result.is_ok() { \"OK\" } else { \"FAIL\" });
        if let Err(message) = &result {
          println!(\"\t\tMessage: {}\", message);
          error += 1;
        } else {
          success += 1;
        }
      "
    done
  done
)
    println!("test summary");
    println!(
      "\t{}\\t{} passed, {} failed",
      if error == 0 { "OK" } else { "FAIL" },
      success, error
    );

    if error > 0 {
      std::process::exit(1);
    }
}
EOF

cd ffi

cargo build --release --quiet

cp target/release/test $output
