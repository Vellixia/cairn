# Security Policy

Cairn is a self-hosted service that stores your project memory and (optionally, in future
releases) shares knowledge between machines. We take its security and privacy posture seriously.

## Reporting a vulnerability

Please **do not** open a public issue for security problems. Instead, report privately via GitHub
Security Advisories ("Report a vulnerability") on the repository, or email the maintainers. We aim
to acknowledge reports within a few days.

## Design principles

- **Local-first, private by default.** Nothing leaves your machine or server unless you explicitly
  enable it. There is no telemetry.
- **Secrets are stripped before they travel.** Any future sharing/federation path runs content
  through a sanitization step (secret + PII redaction) with an explicit consent gate before
  publishing.
- **Lossless by retention.** Compression keeps the full-fidelity original in a content-addressed
  blob store; compressed views are always recoverable rather than destructive.

## Current scope & hardening status

This is early software. Known areas still being hardened (tracked on the roadmap):

- **Auth & multi-device tokens** — bind to `127.0.0.1` for local use; do not expose the server to
  an untrusted network until token auth lands and is enabled.
- **Collective knowledge / federation** — not yet shipped; the sanitization + consent pipeline is
  a prerequisite before any public sharing is enabled.

When in doubt, run Cairn on a trusted network (LAN/VPN such as Tailscale) behind your own
boundary.
