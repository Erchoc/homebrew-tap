class Cb < Formula
  desc "Cross-platform voice assistant that lives in your terminal"
  homepage "https://github.com/Erchoc/chatbot"
  version "0.1.0-beta.6"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/Erchoc/chatbot/releases/download/v#{version}/cb-macos-universal"
      sha256 "9cc997f5b999cf5ac6fb264828629050766ce9ea1b1cd44845fd1f2c858d3b26"
    end

    on_intel do
      url "https://github.com/Erchoc/chatbot/releases/download/v#{version}/cb-macos-universal"
      sha256 "9cc997f5b999cf5ac6fb264828629050766ce9ea1b1cd44845fd1f2c858d3b26"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/Erchoc/chatbot/releases/download/v#{version}/cb-linux-x86_64"
      sha256 "f7d009e8d7db2a09481a1af703ff82c136156560242a559cadeecf3a4c6ac2b4"
    end

    on_arm do
      url "https://github.com/Erchoc/chatbot/releases/download/v#{version}/cb-linux-arm64"
      sha256 "f6ce2535e3c15d0d11b72b4928fedfcf727ff29f9deb99c8277775f65ab3a939"
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
