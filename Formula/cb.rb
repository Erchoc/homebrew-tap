class Cb < Formula
  desc "Cross-platform voice assistant that lives in your terminal"
  homepage "https://github.com/Erchoc/chatbot"
  version "0.1.0-beta.4"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/Erchoc/chatbot/releases/download/v#{version}/cb-macos-universal"
      sha256 "60ec5818a11113666a24fdd45e22523055006d73e581a31a302ba2efd3f655e4"
    end

    on_intel do
      url "https://github.com/Erchoc/chatbot/releases/download/v#{version}/cb-macos-universal"
      sha256 "60ec5818a11113666a24fdd45e22523055006d73e581a31a302ba2efd3f655e4"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/Erchoc/chatbot/releases/download/v#{version}/cb-linux-x86_64"
      sha256 "9de94be549bc01e2162a45a049d5761d5cfa76f78b1d7eb089dd62c7e7a792bf"
    end

    on_arm do
      url "https://github.com/Erchoc/chatbot/releases/download/v#{version}/cb-linux-arm64"
      sha256 "19e6d1acd8acf5f10d15d58ff6a573efec2adccfbfd8e700d7911c1a4931e6ff"
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
