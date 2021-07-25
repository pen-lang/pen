class Pen < Formula
  version '0.1.5'
  desc 'Pen programming language'
  homepage 'https://github.com/pen-lang/pen'
  url "https://github.com/pen-lang/pen/archive/refs/tags/v#{version}.tar.gz"
  sha256 '73a397b9dd2b66b1c07087bf0aa0ae5d322cebf66ad5b41826758d709d011b53'
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
    ENV['RUSTUP_TOOLCHAIN'] = 'stable'

    system (bin / 'pen'), 'create', '.'
    system (bin / 'pen'), 'build'
  end
end
