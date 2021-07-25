class Pen < Formula
  version '0.1.8'
  desc 'Pen programming language'
  homepage 'https://github.com/pen-lang/pen'
  url "https://github.com/pen-lang/pen/archive/refs/tags/v#{version}.tar.gz"
  sha256 '9c5f180d1b9e86b6a2d9a714b83b0fe2817262a6fda15e4cae71f6a05d614c28'
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
