# Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.
# Formula resmi Homebrew untuk Komolang (macOS & Linux)

class Komolang < Formula
  desc "Bahasa pemrograman kompilasi native asli Indonesia berkecepatan tinggi"
  homepage "https://komolang.org"
  url "https://github.com/kangpcode/komolang/archive/refs/tags/v0.1.0.tar.gz"
  version "0.1.0"
  sha256 "0000000000000000000000000000000000000000000000000000000000000000"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "build", "--release", "--bin", "komo"
    bin.install "target/release/komo"
  end

  test do
    assert_match "Komolang", shell_output("#{bin}/komo versi")
  end
end
