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
	sudo \
	wget
run curl -fsSL https://apt.llvm.org/llvm.sh | bash /dev/stdin 14

run useradd -mG sudo penguin
run echo '%sudo ALL=(ALL) NOPASSWD: ALL' >> /etc/sudoers

user penguin
workdir /home/penguin
shell ["bash", "-lc"]

run git clone https://github.com/Homebrew/brew ~/.homebrew
run ~/.homebrew/bin/brew shellenv >> ~/.profile
run brew install hello

run curl -fsSL https://sh.rustup.rs | sh /dev/stdin -y
run cargo install mdbook sccache turtle-build

env LLVM_SYS_140_PREFIX=/usr/lib/llvm-14
env PATH="/usr/lib/llvm-14/bin:$PATH"
