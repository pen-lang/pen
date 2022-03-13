from ubuntu:22.04

run apt update --fix-missing && apt install -y \
	build-essential \
	curl \
	git \
	libssl-dev \
	libz-dev \
	lsb-release \
	pkg-config \
	ruby-bundler \
	ruby-dev \
	software-properties-common \
	wget
run curl -fsSL https://apt.llvm.org/llvm.sh | bash /dev/stdin 14
run curl --proto =https --tlsv1.2 -sSf https://sh.rustup.rs | sh /dev/stdin -y
run git clone https://github.com/pen-lang/pen /root/pen

workdir /root/pen

env LLVM_SYS_130_PREFIX=/usr/lib/llvm-14

run . ~/.cargo/env && cargo build
run . ~/.cargo/env && cargo install --locked --path cmd/pen
run . ~/.cargo/env && cargo install sccache turtle-build

env PEN_ROOT=/root/pen
env PATH="/usr/lib/llvm-14/bin:$PATH"

run . ~/.cargo/env && tools/integration_test.sh
