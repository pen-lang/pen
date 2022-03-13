from ubuntu:22.04

run apt update --fix-missing && apt install -y build-essential \
		curl lsb-release software-properties-common wget
run curl -fsSL https://apt.llvm.org/llvm.sh | bash /dev/stdin 14
run curl --proto =https --tlsv1.2 -sSf https://sh.rustup.rs | sh /dev/stdin -y
run git clone https://github.com/pen-lang/pen /root/pen

workdir /root/pen

run . ~/.cargo/env
run cargo build 
