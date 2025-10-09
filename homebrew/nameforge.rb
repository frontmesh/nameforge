class Nameforge < Formula
  desc "Intelligent photo renaming tool with AI content analysis and GPS location resolution"
  homepage "https://github.com/frontmesh/nameforge"
  url "https://github.com/frontmesh/nameforge/archive/v0.1.0.tar.gz"
  sha256 "REPLACE_WITH_ACTUAL_SHA256"
  license "MIT OR Apache-2.0"
  head "https://github.com/frontmesh/nameforge.git", branch: "master"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    # Test that the binary was installed correctly
    assert_match "nameforge", shell_output("#{bin}/nameforge --help")
    assert_match "nf", shell_output("#{bin}/nf --help")
  end
end