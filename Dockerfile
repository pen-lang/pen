from ubuntu:22.04

run apt update --fix-missing && apt install -y build-essential \
		curl git lsb-release software-properties-common wget
run curl -fsSL https://apt.llvm.org/llvm.sh | bash /dev/stdin 14
run curl --proto =https --tlsv1.2 -sSf https://sh.rustup.rs | sh /dev/stdin -y
run git clone https://github.com/pen-lang/pen /root/pen

workdir /root/pen

env LLVM_SYS_130_PREFIX=/usr/lib/llvm-14

run . ~/.cargo/env && cargo build 
