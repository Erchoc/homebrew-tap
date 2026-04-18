class Cb < Formula
  desc "Cross-platform voice assistant that lives in your terminal"
  homepage "https://github.com/Erchoc/chatbot"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/Erchoc/chatbot/releases/download/v#{version}/cb-macos-universal"
      sha256 "43b9860abe5027876474b85f3dfa1783bbd15a0516ff084d859ddef77f5fec88"
    end

    on_intel do
      url "https://github.com/Erchoc/chatbot/releases/download/v#{version}/cb-macos-universal"
      sha256 "43b9860abe5027876474b85f3dfa1783bbd15a0516ff084d859ddef77f5fec88"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/Erchoc/chatbot/releases/download/v#{version}/cb-linux-x86_64"
      sha256 "75fc49dafdaf9a3720b3ab328d60ec9b04f67bbed5befed2ea6b3ad728e8381b"
    end

    on_arm do
      url "https://github.com/Erchoc/chatbot/releases/download/v#{version}/cb-linux-arm64"
      sha256 "c50ef174ce01aaf77dcd4503915de00df2bcea904002e6240145ce7258a08aac"
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
