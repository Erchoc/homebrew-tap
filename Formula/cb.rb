class Cb < Formula
  desc "Cross-platform voice assistant that lives in your terminal"
  homepage "https://github.com/Erchoc/chatbot"
  version "0.1.0-beta"
  license "MIT"

  on_macos do
    url "https://github.com/Erchoc/chatbot/releases/download/v#{version}/cb-macos-universal"
    sha256 "3919f2d5b2133229363caa6a8fd52b8ba419cdf107b4ddfc69751d060175dfe0"
  end

  on_linux do
    on_intel do
      url "https://github.com/Erchoc/chatbot/releases/download/v#{version}/cb-linux-x86_64"
      sha256 "c3066647610bbbc3afc59b7e08f7148e4690a1821762558a6ea6e776fa711bfd"
    end

    on_arm do
      url "https://github.com/Erchoc/chatbot/releases/download/v#{version}/cb-linux-arm64"
      sha256 "74577037765712f1431054c850fe5c404b603e85d80c1948f36a26fe2e5f979a"
    end
  end

  def install
    binary = stable.url.split("/").last
    bin.install binary => "cb"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/cb --version")
  end
end
