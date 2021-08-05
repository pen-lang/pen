class Pen < Formula
  version '0.1.12'
  desc 'Pen programming language'
  homepage 'https://github.com/pen-lang/pen'
  url "https://github.com/pen-lang/pen/archive/refs/tags/v#{version}.tar.gz"
  sha256 '2e0d9ca41db76d4801726bc0190bd07206049ceba5c6a315b402b5e48d0b693c'
  license 'MIT'

  conflicts_with 'pen'

  depends_on 'git'
  depends_on 'llvm@12'
  depends_on 'ninja'
  depends_on 'rust' => :build

  def install
    system 'cargo', 'build', '--locked', '--release'
    libexec.install 'target/release/pen'

    File.write 'pen.sh', <<~EOS
      #!/bin/sh
      set -e
      export PEN_ROOT=#{prefix}
      export PATH=#{Formula['llvm@12'].opt_bin}:$PATH
      #{libexec / 'pen'} "$@"
    EOS

    chmod 0o755, 'pen.sh'
    libexec.install 'pen.sh'
    bin.install_symlink (libexec / 'pen.sh') => 'pen'

    lib.install Dir['lib/*']
  end

  test do
    ENV.prepend_path 'PATH', Formula['rust'].opt_bin
    ENV.prepend_path 'PATH', bin

    system 'pen', 'create', '.'
    system 'pen', 'build'
    system './app'
  end
end
