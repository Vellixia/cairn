# Changelog

All notable changes to Cairn are documented here. Versions follow [Semantic Versioning](https://semver.org/).

## [0.5.0] — 2026-06-20 — Context + Reliability Layer (Phase 3.5 + 4.0)

### What's new

**Memory & confidence (Sprint 2–3)**
- `confidence: f32` + `pinned: bool` on every memory; reinforced by the
  agentmemory curve `c' = min(1.0, c + 0.1*(1-c))` on every access.
- Provenance edges on `Memory`: `derived_from`, `contradicts`, `supersedes`,
  `applies_to`. New `/dashboard/memory/graph` page with a pure-SVG force layout.
- `MemoryEngine::crystallize()` promotes a working-tier cluster to a semantic-tier
  crystal (agentmemory's "lesson" pattern).

**Reliability (Sprint 4–5)**
- New `cairn-session` crate owns session + drift JSONL storage and
  approve/reject workflow. `/dashboard/sessions` + `/dashboard/reliability/drift`
  pages.
- HMAC-SHA256-signed ledger at `<data_dir>/ledger.jsonl` for every context
  assembly. `/api/ledger` + `/api/ledger/verify` expose the chain.
- `/dashboard/savings` page renders the per-assemble savings breakdown.

**Audit + observability (Sprint 1)**
- Audit events are now durable HelixDB records (was in-memory ring); the
  `/api/events` SSE stream uses `Last-Event-ID` replay from durable storage
  instead of 5 s polling. `/api/metrics` exposes the live counters.

**Hybrid search (Sprint 7)**
- `MemoryEngine::hybrid_search()` combines lexical (BM25-lite) + semantic
  via Reciprocal Rank Fusion; MMR diversity rerank (`λ=0.7`) keeps the top-N
  non-redundant. Exposed as `/api/search` and `cairn-cli search`.

**CLI surface (Sprint 9–10)**
- 25 new MCP tools (`memory_edit`, `memory_delete`, `memory_pin`,
  `memory_reinforce`, `memory_timeline`, `memory_crystallize`, `memory_graph`,
  `graph`, `search`, `metrics`, `stats`, etc.). Total tool count is now 40+.
- New CLI subcommands: `cairn-cli graph related|impact|callgraph`,
  `memory timeline|crystallize`, `search`, `sessions`, `session`, `metrics`.

**Zero-prompt setup (Sprint 8)**
- `cairn-cli onboard` runs `doctor --fix` + provisions the local store + wires
  every detected agent in one shot. `cairn-cli doctor --fix` repairs missing
  data dirs, weak MinIO creds, etc. Non-zero exit when remediation is required.

**Context packages (Sprint 11)**
- `.cairnpkg` format: tarball with `manifest.json` + `memory.jsonl` +
  `profile.jsonl` + `patterns.jsonl` + `graph.jsonl` + `signature.sha256`.
  Per-file SHA-256 + HMAC signature; rejects oversized (>16 MiB) and tampered
  packs. `.ctxpkg` is accepted as an import alias.
- New `cairn-pack` crate + `cairn-cli pack` with 9 actions:
  `create | info | install | list | remove | export | import | auto-load |
  publish`.

**Distribution polish (Sprint 12)**
- **Homebrew tap** at `Vellixia/homebrew-tap` (`brew install Vellixia/tap/cairn`).
- **One-click deploys** for Fly.io (`deploy/fly.toml`), Railway
  (`deploy/railway.toml`), and Render (`deploy/render.yaml`).
- **Non-root Docker volume init.** New `cairn-init` service chowns `/data` to
  uid 10001 before `cairn` starts as non-root. The pre-0.5.0 `user: "0"`
  workaround is gone.
- **README OpenCode quickstart** section.

### Security

- Web dashboard ships a **per-request CSP nonce** (random 16 bytes per
  response, injected into `<script>` tags). Closes the static-`script-src`
  gap that would otherwise block the v0.5.0 interactive pages.
- **Setup wizard v2** (`/setup/wizard`) replaces the original `/setup` flow
  with a 4-step admin → embed → pair → health walkthrough. v1 `/setup` is
  retained as a fallback with a deprecation banner.
- **HMAC-SHA256 ledger** detects tamper attempts on the savings record.

### Test count

`cargo test --workspace` reports **225 passed, 5 ignored, 0 failed**
as of this release (up from 118 in 0.3.0). The 5 ignored tests require a
live HelixDB.

See [ADR-010 through ADR-016](docs/DECISIONS.md) for the full reasoning behind
each decision.

---

## [0.3.0] — 2026-06-19 — P0–P3 Security & Build Hardening

### Breaking changes

- **CLI binary split.** The single `cairn` binary was replaced by two
  binaries: `cairn` (the server: `serve`, `token`, `pair-code`) and
  `cairn-cli` (client commands: `setup`, `mcp`, `hook`, `sync`, `bench`,
  `pair`, `update`, `rule`). The `cairn install <agent>` subcommand was
  removed; use `cairn-cli setup <agent>`. User scripts that invoke
  `cairn install` must be updated.

- **Device tokens are now signed JWTs (HS256), not opaque bearer
  values.** Previously-issued plaintext tokens are invalid after upgrade
  to this release. Re-mint each device token:
  ```sh
  cairn-cli token create --name <device> --scope <admin|write|read>
  ```
  The bearer value is shown exactly once. The server stores only token
  id, name, scope, and created_at; the JWT itself is regenerated from
  those fields + `CAIRN_SECRET_KEY` on each request.

- **`CAIRN_SECRET_KEY` is now required and must be ≥ 32 bytes.** The
  server fails to start if the env var is missing, empty, or too short.
  Generate one with:
  ```sh
  openssl rand -base64 48
  ```
  Set it in `.env` or `~/.config/cairn/.env`. Existing deployments that
  boot without `CAIRN_SECRET_KEY` will refuse to start.

- **TLS required for non-loopback binds.** `cairn serve` on a non-loopback
  address (`0.0.0.0`, LAN IP, DNS name) now refuses to start unless both
  `CAIRN_TLS_CERT` and `CAIRN_TLS_KEY` are set. Set
  `CAIRN_INSECURE=1` for trusted local/private networks only.

- **Docker compose default port bind changed.** The bundled stack now
  binds to `127.0.0.1:7777` instead of `0.0.0.0:7777`. To expose on the
  LAN, override with `-p "0.0.0.0:${CAIRN_PORT:-7777}:7777"`.

- **`CAIRN_CORS_ORIGINS=*` is now rejected.** Set explicit origins
  instead. Falling back to same-origin-only CORS for the wildcard case.

### Security

- JWT device tokens (HS256, 32+ byte secret requirement, id-based revoke)
- Workspace root boundary enforcement in `ContextEngine` and MCP
- TLS enforcement for non-loopback binds
- Default MinIO credentials removed; `minio-guard` service fails fast
  on weak/empty credentials
- Install script SHA256SUMS verification + SLSA provenance check
- SLSA Level 3 provenance + keyless Sigstore cosign signing on releases
- Profile sanitization (escape, strip, wrap directive-delimiter blocks)
- Hashed preference storage with `suspicious` flag

### Build & CI

- Workspace dependencies pinned to specific minors via tilde
  (`~major.minor`) with `cargo build --locked` enforced in CI
- `cargo-audit` and `cargo-deny` added to CI (`.github/workflows/rust-security.yml`)
- GitHub Actions SHA-pinned across all workflows
  (ci, rust-security, release); Dependabot weekly digest
- Install scripts: SHA256SUMS + optional cosign SLSA provenance
  verification (soft gate by default; `CAIRN_INSTALL_REQUIRE_ATTESTATION=1`
  for hard gate)

### Test count

`cargo test --workspace` reports **118 passed, 5 ignored, 0 failed**
as of this release (up from 87 before hardening; the 5 ignored require
a live HelixDB).