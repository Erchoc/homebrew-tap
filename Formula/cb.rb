class Cb < Formula
  desc "Cross-platform voice assistant that lives in your terminal"
  homepage "https://github.com/Erchoc/chatbot"
  version "0.1.0-beta.5"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/Erchoc/chatbot/releases/download/v#{version}/cb-macos-universal"
      sha256 "6e399d8ab272bbcaa0adacf3dbaac63111b3631612061a9a7d70244ce3e3b2be"
    end

    on_intel do
      url "https://github.com/Erchoc/chatbot/releases/download/v#{version}/cb-macos-universal"
      sha256 "6e399d8ab272bbcaa0adacf3dbaac63111b3631612061a9a7d70244ce3e3b2be"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/Erchoc/chatbot/releases/download/v#{version}/cb-linux-x86_64"
      sha256 "e151ff1714716511f465b3e6ccb8d29997a53d121f0a55019f7db30d7cd04898"
    end

    on_arm do
      url "https://github.com/Erchoc/chatbot/releases/download/v#{version}/cb-linux-arm64"
      sha256 "a06427d656d8bfc3a2c68be49299d85e1f7ce0a3dbe3c4b24ca99634ec59a48c"
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
