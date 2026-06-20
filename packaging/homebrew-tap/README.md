# Vellixia Homebrew tap

Homebrew formulae for [Vellixia](https://github.com/Vellixia) projects.

## Install Cairn

```sh
brew tap Vellixia/tap
brew install cairn
```

Then start the stack and connect an agent:

```sh
docker compose up -d                       # HelixDB + MinIO + Cairn server on :7777
cairn-cli setup opencode --server http://localhost:7777
```

See the [Cairn README](https://github.com/Vellixia/Cairn#quick-start) for the full
quickstart.

## Updating the `cairn` formula

The `cairn.rb` formula lives at `Formula/cairn.rb`. To release a new version:

1. Tag + push a GitHub release (`vX.Y.Z`) — the GitHub Actions release workflow
   builds + uploads `cairn-<target>.tar.gz` + `SHA256SUMS`.
2. In this tap repo, bump `version` in `Formula/cairn.rb`.
3. Replace each `REPLACE_ME_AT_RELEASE_TIME_*` placeholder with the corresponding
   line from the new release's `SHA256SUMS` file.
4. Run `brew audit --new --strict cairn` locally to validate.
5. Open a PR; CI (`brew test cairn`) verifies the binary boots on macOS + Linux.

The tap repo is intentionally separate from the main Cairn repo so that
`brew update` doesn't have to pull Rust toolchain metadata.
