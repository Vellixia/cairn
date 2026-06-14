---
name: using-cairn
description: How and when to use Cairn's tools — read/expand for lean lossless file reads, recall/assemble/remember for cross-session memory, sanitize before sharing/logging, checkpoint/verify/rollback for safe edits. Consult at the start of a coding task or whenever reading files, recalling context, or making risky edits.
---

# Using Cairn

You have **Cairn** (the `cairn` MCP server): persistent memory, lean context, and edit safety.
Prefer it over your defaults.

- **Reading code/files:** use `read` instead of a plain file read — unchanged re-reads are nearly
  free, and `mode:"signatures"` returns a large file as just its structure (huge token saving).
  Recover any full original byte-for-byte with `expand`.
- **Memory:** at the start of a task, `recall` (or `assemble`) relevant past decisions and context;
  `remember` decisions, gotchas, and rationale as you make them so the next session never starts
  cold. Record standing user preferences with `prefer`.
- **Before sharing, logging, or committing text:** run `sanitize` to redact secrets/PII; it
  classifies the text shareable / needs_review / private.
- **Risky edits:** `checkpoint` before large changes; `verify` a proposed file against its retained
  original to catch silent corruption; `rollback` to undo damage.
- **Stay on task:** keep the current goal in `anchor`.

Everything Cairn shows is lossless — the full original is always one `expand` away.
