# P0: JWT device tokens + build fix

This PR implements the first P0 security fix from the security audit: replacing plaintext device
strings with signed JWTs and removing them from the backend.

## What changed

- `cairn-core/src/model.rs`
  - `DeviceToken` now has `id` (the stored identifier), `name`, `created_at`, and an optional
    `token` field that is only present when the token is first minted.

- `cairn-core/src/config.rs`
  - Added `secret_key: Option<Vec<u8>>` backed by `CAIRN_SECRET_KEY`.

- `cairn-api/src/auth.rs` (new)
  - `TokenSigner`: HS256 JWT signer/verifier.
  - `extract_bearer` helper for the `Authorization` header.

- `cairn-api/src/lib.rs`
  - `AppState` carries an optional signer and exposes `sign_token`, `verify_bearer`, and
    `revoke_bearer`.
  - `pair_new` / `pair_claim` now mint a JWT on the server side.
  - The auth middleware verifies the JWT signature before checking the stored token id.

- `cairn-store/src/db.rs` + `cairn-store/src/helix.rs`
  - `create_token`, `validate_token_id`, `revoke_token`, `list_tokens` now work with token ids, not
    raw secrets.
  - Pairing tables store `token_id` instead of the bearer.
  - Tests updated to validate by id.

- `cairn-cli/src/main.rs` + `cairn-cli/src/pair.rs`
  - `cairn token create` prints the freshly minted JWT.
  - `cairn token list` prints the token id (never the JWT).
  - `cairn token revoke` decodes the JWT to extract the id before revoking.
  - `cairn pair` registers the new token id with the pairing code.

- `.env.example`
  - Documented `CAIRN_SECRET_KEY`.

- Build fix from previous PR (merged)
  - `web/out/.gitkeep` + `crates/cairn-api/build.rs`.

## How to test

```bash
export CAIRN_SECRET_KEY="my-super-secret-key-of-32-or-more-bytes"
export CAIRN_HELIX_URL=http://localhost:6969
cargo test -p cairn-store
cargo test -p cairn-api pair_new_then_claim_yields_a_valid_token_once
cargo check --workspace
```

## Remaining P0 follow-up

- TLS enforcement when binding to non-loopback addresses.
- Install script checksum verification.
- Default MinIO credentials removal.
- MCP workspace root boundary.
- Preference injection hardening.

Refs: docs/P0_SECURITY_PLAN.md, docs/PLAN.md section on auth.
