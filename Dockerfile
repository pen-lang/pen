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
run useradd -m penguin

user penguin

run git clone https://github.com/Homebrew/brew ~/.homebrew
run ~/.homebrew/bin/brew shellenv >> ~/.profile
run . ~/.profile && brew install hello

run curl -fsSL https://sh.rustup.rs | sh /dev/stdin -y
run . ~/.profile && cargo install mdbook sccache turtle-build

copy . /home/penguin/pen

env LLVM_SYS_130_PREFIX=/usr/lib/llvm-14
env PEN_ROOT=/home/penguin/pen
env PATH="/usr/lib/llvm-14/bin:$PATH"

workdir /home/penguin/pen

run . ~/.profile && tools/build.sh
run . ~/.profile && tools/lint.sh
run . ~/.profile && tools/format.sh
run . ~/.profile && tools/unit_test.sh
run . ~/.profile && tools/integration_test.sh
run . ~/.profile && tools/build_documents.sh
