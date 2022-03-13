from ubuntu:18.04

run apt update --fix-missing && apt install -y build-essential curl
run curl -fsSL https://apt.llvm.org/llvm.sh | bash /dev/stdin 14
