# Homebrew formula for the `cairn` and `cairn-cli` binaries.
#
# Lives at https://github.com/Vellixia/homebrew-tap/blob/main/Formula/cairn.rb
# (the tap repo is `Vellixia/homebrew-tap` per the v0.5.0 plan §Phase 4.0 §12).
#
# Install with:
#   brew tap Vellixia/tap
#   brew install cairn
#
# The formula ships both binaries (cairn-server + cairn-cli) under the same
# `cairn` formula because they are released together from the same GitHub release
# tarball (cairn-<target>.tar.gz). Splitting them into two formulae would require
# two downloads and two version pins to keep in lock-step.

class Cairn < Formula
  desc "Cairn — context + reliability layer for AI agents (server + CLI)"
  homepage "https://github.com/Vellixia/Cairn"
  version "0.5.0"
  license "Apache-2.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Vellixia/Cairn/releases/download/v#{version}/cairn-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_ME_AT_RELEASE_TIME_AARCH64_APPLE_DARWIN"
    else
      url "https://github.com/Vellixia/Cairn/releases/download/v#{version}/cairn-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_ME_AT_RELEASE_TIME_X86_64_APPLE_DARWIN"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/Vellixia/Cairn/releases/download/v#{version}/cairn-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_ME_AT_RELEASE_TIME_AARCH64_LINUX_GNU"
    else
      url "https://github.com/Vellixia/Cairn/releases/download/v#{version}/cairn-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_ME_AT_RELEASE_TIME_X86_64_LINUX_GNU"
    end
  end

  # `cairn` is the server; `cairn-cli` is the client. We install both into the
  # Homebrew bin directory so `cairn serve` and `cairn-cli setup` are both on PATH.
  def install
    bin.install "cairn"
    bin.install "cairn-cli"
  end

  # Smoke test: run `--version` on both binaries so a broken release gets caught at
  # `brew install` time rather than at the user's first invocation.
  test do
    assert_match "cairn #{version}", shell_output("#{bin}/cairn --version")
    assert_match "cairn-cli #{version}", shell_output("#{bin}/cairn-cli --version")
  end

  # Caveats shown to the user after install. Encourages them to either run
  # `docker compose up` (the easy path) or set CAIRN_HELIX_URL pointing at a
  # managed HelixDB — exactly the same flow as the README quickstart.
  def caveats
    <<~EOS
      Cairn stores memory in HelixDB. To run the full stack locally:
        docker compose up -d   # starts HelixDB + MinIO + Cairn on :7777

      Or point CAIRN_HELIX_URL at an existing HelixDB instance and run:
        cairn serve

      Then configure your AI agent:
        cairn-cli setup opencode --server http://localhost:7777
    EOS
  end
end
