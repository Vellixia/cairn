# Contributing to Cairn

Thanks for your interest! Cairn is an early-stage, open-source project and contributions are
welcome.

## Development setup

You'll need a recent **Rust** toolchain (stable, MSRV 1.85) and **Node 20+** (for the web UI).
A running **HelixDB** instance is required for live integration tests â€” the simplest path is
`docker compose up -d helix`.

```sh
# engine
cargo build --workspace
cargo test --workspace

# run the server (+ embedded UI / built-in fallback page) on http://127.0.0.1:7777
cargo run -p cairn-server -- serve

# web control plane (dev server on http://localhost:3000, talks to the API on :7777)
cd web && npm install && npm run dev
```

The Rust build does **not** require building the web UI â€” `crates/cairn-api/build.rs`
creates `web/out/` at compile time when missing, so the binary falls back to a
built-in page when no export is present. Build artifacts are never committed.

## Before you open a PR

CI runs these â€” please run them locally first:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- Keep changes focused; one logical change per PR.
- Match the surrounding style. New behavior gets a test.
- Dependencies use tilde constraints (`~major.minor`); build with `--locked` to catch drift.
- CI runs `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `cargo build --workspace` on every PR via `.github/workflows/ci.yml`. Run the same commands locally before pushing.

## Workspace layout

22 crates. Dep graph: `cairn-core` â†’ `cairn-store` â†’ domain crates â†’ `cairn-mcp` â†’ `cairn-api`
â†’ `cairn-server` / `cairn`.

| Crate | Role |
|---|---|
| `cairn-core` | domain types, hashing, config, `OrgId` tenant scope |
| `cairn-store` | pluggable backend (HelixDB + in-memory) + content-hash blob store |
| `cairn-context` | cached reads Â· AST signature outlines Â· byte-identical `expand` |
| `cairn-memory` | remember Â· BM25/semantic recall Â· wakeup Â· decay Â· 4-tier consolidation Â· MMR hybrid |
| `cairn-assemble` | token-budgeted, edge-ordered context assembler with CSP nonce support |
| `cairn-guard` | verify edits vs originals Â· task anchor Â· checkpoint/rollback Â· reliability score |
| `cairn-shell` | RTK-style command-output compression (lossless via `expand`) |
| `cairn-profile` | preference learning |
| `cairn-share` | privacy-first sanitization (redact secrets/PII before sharing) |
| `cairn-session` | on-disk session store (JSON), replay + resume |
| `cairn-pack` | `.cairnpkg` archive format (ustar + SHA-256 + HMAC + Ed25519) |
| `cairn-registry` | Ed25519-signed pack registry Â· trust scopes Â· revocation cascade |
| `cairn-sync` | CRDTs (GCounter + ORSet) Â· vector clocks Â· E2E encryption (Argon2id + ChaCha20-Poly1305) |
| `cairn-bench` | LongMemEval / horizon / retention benchmarks |
| `cairn-proactive` | intent classifier Â· auto-inject Â· opt-out (`PROJECT_OPT_OUT`) |
| `cairn-proxy` | `cairn.sh` reverse proxy Â· fanout Â· claim tokens |
| `cairn-ingest` | VTT/SRT/JSON transcript parsers Â· speaker-window chunking |
| `cairn-embed` | embedding providers (hashing default, ONNX opt-in) |
| `cairn-api` | axum REST API Â· embedded web UI Â· PWA service worker Â· push subscriptions |
| `cairn-mcp` | MCP server (stdio) Â· 29 tools + 10 graph actions = 39 total Â· 6 resources Â· 5 prompts |
| `cairn` | the `cairn` binary (serve, mcp, setup, hook, sync, bench, token, pack, sync, proxy) |
| `cairn-server` | the `cairn` binary entry point (alternative name) |

The two binaries shipped are `cairn` (from `cairn-server`) and `cairn`.

## License

By contributing, you agree your contributions are licensed under the project's
[Apache-2.0](LICENSE) license.
