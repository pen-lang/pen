class Pen < Formula
  version '0.1.4'
  desc 'Pen programming language'
  homepage 'https://github.com/pen-lang/pen'
  url "https://github.com/pen-lang/pen/archive/refs/tags/v#{version}.tar.gz"
  sha256 '555161445bbecbaa646c144d47c64c993906e224a6bcdf9538c8c1e0fedf0ba3'
  license 'MIT'

  conflicts_with 'pen'

  depends_on 'llvm'
  depends_on 'rust' => :build

  def install
    system 'cargo', 'install', *std_cargo_args.map { |s| s == "." ? "cmd/pen" : s }
  end

  test do
    system "#{bin}/pen", '--version'
  end
end
