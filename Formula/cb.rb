class Cb < Formula
  desc "Cross-platform voice assistant that lives in your terminal"
  homepage "https://github.com/Erchoc/chatbot"
  version "0.1.2"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/Erchoc/chatbot/releases/download/v#{version}/cb-macos-universal"
      sha256 "a52b26b3b05aebf8dd8172d6aee820ae19b9ba27532c786f4c00d26150c4f148"
    end

    on_intel do
      url "https://github.com/Erchoc/chatbot/releases/download/v#{version}/cb-macos-universal"
      sha256 "a52b26b3b05aebf8dd8172d6aee820ae19b9ba27532c786f4c00d26150c4f148"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/Erchoc/chatbot/releases/download/v#{version}/cb-linux-x86_64"
      sha256 "f8554fe8107e9a816cf6ed8c90217981cb0fc86545b50407f3f734f8ecd75f29"
    end

    on_arm do
      url "https://github.com/Erchoc/chatbot/releases/download/v#{version}/cb-linux-arm64"
      sha256 "8bc4eb24f696c696c5255dcf3b0a53be2859d73923c7c48b6ba73ee91758fba2"
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
