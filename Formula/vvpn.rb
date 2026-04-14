class Vvpn < Formula
  desc "Stash VPN 环境诊断工具"
  homepage "https://github.com/Erchoc/homebrew-tap"
  url "https://github.com/Erchoc/homebrew-tap/releases/download/v0.1.0/vvpn-0.1.0-darwin-universal.tar.gz"
  sha256 "PLACEHOLDER"
  version "0.1.0"
  license "MIT"

  def install
    bin.install "vvpn"
  end

  test do
    assert_match "vvpn", shell_output("#{bin}/vvpn 2>&1", 0)
  end
end
