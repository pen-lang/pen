from ubuntu:20.04

env DEBIAN_FRONTEND=noninteractive

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
run curl -fsSL https://apt.llvm.org/llvm.sh | bash /dev/stdin 16

run useradd -mG sudo penguin
run echo '%sudo ALL=(ALL) NOPASSWD: ALL' >> /etc/sudoers

user penguin
workdir /home/penguin
shell ["bash", "-lc"]

run git clone https://github.com/Homebrew/brew ~/.homebrew
run ~/.homebrew/bin/brew shellenv >> ~/.profile
run brew info hello

run curl -fsSL https://sh.rustup.rs | sh /dev/stdin -y
run cargo install mdbook sccache turtle-build

env LLVM_SYS_160_PREFIX=/usr/lib/llvm-16
env PATH="/usr/lib/llvm-16/bin:$PATH"
