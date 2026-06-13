# Cairn — The Open-Source Context & Reliability Layer for AI Agents

> **Make any model smart.** Remember everything · feed less, not more · stay reliable on long
> tasks · get smarter together — self-hosted, one Rust binary, with no context ever lost.

---

## Context

Using AI coding agents (Claude Code, Codex, OpenCode, Cursor…) over long or multi-session work
fails in ways that bigger context windows do **not** fix. The user named two pains (forgetting,
re-reading); 2026 research shows the problem is deeper and partly *architectural*. Cairn is a
new, from-scratch **Rust** tool that unifies the best ideas of four references into one
self-hostable engine, and addresses **all** of the failure classes below.

### The four references (ideas only — no forking)

| Project | Lang | What we take |
|---|---|---|
| **agentmemory** (rohitg00) | TS/iii | 4-tier memory (working→episodic→semantic→procedural), consolidation/decay, hybrid recall (BM25+vector+graph, RRF), lifecycle hooks, viewer |
| **lean-ctx** (yvgude) | Rust | 10 read modes, ~13-tok cached re-reads, tree-sitter AST (18 langs), property graph, `serve` HTTP-MCP |
| **rtk / Rust Token Killer** (rtk-ai) | Rust | Command-output compression (filter/group/dedup) via hooks, 100+ filters, **tee/recover**, single binary, ~80% session cut — proves the Rust+hook approach |
| **caveman** (JuliusBrussee) | JS/Py | Output-**style** compression (~65% output cut, reasoning intact); finding that brevity can *raise* accuracy |

**Name:** **Cairn** — a stack of trail-marker stones. Travelers each add a stone, everyone who
follows benefits (**collective knowledge**); each session leaves a marker the next one follows
(**memory**); a cairn is minimal — only the stones needed to navigate (**lean, no-loss context**).

---

## Decisions locked (from Q&A)

- **Build:** brand-new tool in **Rust**; all four projects are references only.
- **Audience:** **open-source, self-hosted** — public repo, many self-hosters, **federation**
  between servers, **high polish + a high privacy/sanitization bar.**
- **Posture:** **active guardrails** — Cairn doesn't just feed context, it *verifies* agent
  output against retained originals, detects drift, and re-anchors long tasks. (Not full
  multi-agent orchestration in v1 — see Scope.)
- **Hero:** *all of it* — five pillars, unified. Umbrella message: "the context & reliability
  layer." Punch line: "make any model smart."
- **First milestone:** brand + landing + dashboard **and** live server + sync, in parallel.
- **Name:** Cairn (backups: Mnemo, Marrow).

---

## Deep Problem Analysis (2026 research)

**The throughline: the bottleneck is the *context fed to the model* and the *drift over time* —
not the model's IQ.** That is exactly the gap Cairn fills, and why "make a dumb model smart" is
achievable. Failures group into five classes:

**A. Context-window failures (architectural — bigger windows don't help):**
- **Context rot** — Chroma tested 18 frontier models; *all* degrade as input grows, even on
  trivial tasks; quality "drops off a cliff" past ~50% fill. Cause is attention math (RoPE decay
  + softmax), not retrieval. → *feed less + ordered, don't just compress.*
- **Lost-in-the-middle** — mid-context info is ignored; models favor the edges. → *put critical
  facts at the start/end.*
- **Lossy compaction** — auto-summarization silently drops decisions, constraints, gotchas.

**B. Long-horizon failures (temporal — the active-guardrails target):**
- **Error compounding / reasoning drift** — small per-step errors snowball to systematic
  failure; silent, "no stack trace, no alert" (Microsoft: models "can't handle long-running tasks").
- **Silent corruption when delegating** — frontier models lost ~25% of document content over 20
  delegated edits. → *verify output against a retained ground truth.*
- **Reliability ≠ pass@1** — production needs reliability across many steps.

**C. Continuity failures (state):** cross-session amnesia · cross-device silos · cross-agent
silos · re-reading unchanged files · re-explaining conventions · lost rationale · no task resume.

**D. Knowledge/capability failures (collective):** weak/cheap models underperform for lack of
context (not IQ) · knowledge siloed per user · everyone re-learns the same lessons · the agent-
memory market is hot but **memory-only** (MemPalace, mem0…) — no unified, self-host, no-loss,
reliable, collective solution.

**E. Multi-agent (scoped out of v1):** 41–86% failure rates, mostly *coordination*, not
capability; "context collapse" is the top long-task killer; single-agent often *beats*
multi-agent. → Cairn targets **single-agent reliability first**; shared-context primitives later.

---

## Core Principles

1. **No context loss — lossless by retention.** Cairn is a stateful server: every compression
   (file, shell output, response, memory) **retains the full-fidelity original** in a content-
   hash blob store. The agent gets a **compressed view + a handle**; any view is **expandable on
   demand** (`expand`/`recover`). Window shrinks 60–90%; the system loses nothing. (Beats rtk's
   stateless re-exec and caveman's irreversible loss.)
2. **Less, not more — anti-rot.** Don't dump context. **Assemble** the minimal, highest-signal,
   best-*ordered* working set under a token budget; critical facts at the edges; the rest one
   `expand` away.
3. **Private by default.** Nothing leaves a device/server without explicit, sanitized, revocable
   opt-in. Essential for an open-source, federated, collective product.

---

## Product Vision — five pillars

1. **Remember** — never start cold; decisions/tasks/rationale persist across sessions, devices, agents.
2. **Compress without loss** — files, shell output, responses shrink in the window, stay fully recoverable.
3. **Assemble lean context** — fight context rot: feed less, higher-signal, well-ordered context.
4. **Stay reliable** — verify edits vs originals, detect drift, re-anchor long tasks (active guardrails).
5. **Get smarter together** — learn each user's preferences + opt-in **collective knowledge** so
   cheap/small models behave like senior, personalized engineers.

**Differentiation / moat:** most agent-memory tools are memory-only, cloud/library, Python.
Cairn unifies **memory + no-loss compression + anti-rot assembly + active reliability + collective
federation** as one self-hostable **Rust** binary. The *integration* is the moat.

**Benchmark plan (publish, honestly, as targets→measured):** LongMemEval + LoCoMo (recall),
token-reduction on a standard session (target 60–90%), **byte-identical** expand/recover,
task-success lift at increasing horizons (drift), all in CI.

---

## Branding

- **Name:** Cairn. **Hero:** *"Make any model smart."* **Umbrella:** *"The open-source context &
  reliability layer for AI agents."*
- **Logo:** minimal 3-stone stack doubling as graph nodes; top stone = accent ("trail blaze").
- **Palette:** Ink `#0B0F14` · Surface `#12181F` · Slate `#8A94A6` · Off-white `#ECEFF4` · Accent
  ember `#FB923C` · Signal teal `#2DD4BF`.
- **Type:** Geist Sans + Geist Mono (Inter + JetBrains Mono fallback).
- **Voice:** precise, calm, wayfinding — "leave a marker," "only the stones you need," "never
  start cold," "every traveler adds a stone."

---

## Architecture

```
 Devices/Agents (Claude Code, Codex, OpenCode, Cursor, Windsurf, Gemini CLI...)
   │  one MCP endpoint (stdio shim OR HTTP) + per-device token
   │  + lifecycle hooks (SessionStart, Pre/PostToolUse, PreCompact, SessionEnd)
   ▼
 ┌──────────────────────── Cairn server (one Rust binary) ─────────────────────────┐
 │ cairn-mcp (curated ~26 tools)        cairn-api (axum REST+WS, auth, tokens)      │
 │ ┌─────────┐┌────────┐┌────────┐┌────────┐┌────────┐┌──────────┐┌──────────────┐ │
 │ │ context ││ shell  ││ memory ││ profile││ guard  ││collective││    sync      │ │
 │ │ read +  ││100+ cmd││4 tiers,││ learns ││ verify ││ opt-in,  ││ offline-first│ │
 │ │ cache + ││compress││ decay, ││ user   ││ drift, ││ sanitized││ CRDT, E2E    │ │
 │ │ ASSEMBLE││+recover││ dedup  ││ prefs  ││re-anchor││federate ││              │ │
 │ └────┬────┘└───┬────┘└───┬────┘└───┬────┘└───┬────┘└────┬─────┘└──────┬───────┘ │
 │      └ search (BM25+vector+graph, RRF+rerank+MMR) ─ graph (property+temporal) ┘ │
 │      blob-store: full-fidelity originals (content-hash) ── EXPAND/RECOVER any view│
 │      cairn-store: SQLite+sqlite-vec (local) │ Postgres+pgvector (server)          │
 └───────────────────────────────────────────────────────────────────────────────────┘
   ▲ web dashboard + landing (Next.js, embedded via rust-embed)   ▲ federation: signed,
   ▼ browser                                                      ▼ sanitized knowledge packs
```

### Cargo workspace (crates)

- `cairn-core` — domain types, config, errors; memory/context/profile/knowledge/reliability model.
- `cairn-store` — storage + **full-fidelity blob store** (content-hash originals). SQLite +
  `sqlite-vec` (local); Postgres + `pgvector` (server); `sqlx`.
- `cairn-context` — read modes, content-hash+mtime **cache** (~13-tok re-reads, diff-only after
  edits), **tree-sitter** AST, optional response-style compressor (caveman, opt-in), and the
  **Context Assembler** (below).
- `cairn-shell` — rtk-style command-output compression (100+ filters); originals to blob store → **recover**.
- `cairn-memory` — 4 tiers, consolidation + Ebbinghaus decay + eviction, SHA-256 dedup, contradiction detection.
- `cairn-profile` — **preference/behavior learning** (stack, style, libs, do/don'ts, corrections,
  tone); injected at session start + relevant moments. *(The "make dumb models smart" engine.)*
- `cairn-guard` — **active guardrails** (below): verification vs originals, drift detection,
  re-anchoring, checkpoints, reliability scoring.
- `cairn-collective` — opt-in shared knowledge: distilled, **sanitized** units; private/team/
  public pools; consent-gated publish; trust/voting/provenance/decay; **federation + signed packs**.
- `cairn-search` — hybrid retrieval (`tantivy` BM25 + vector + graph), RRF + rerank + **MMR**
  diversity; recall fuses memory + profile + collective with provenance.
- `cairn-graph` — property graph (imports/calls/exports/type-refs → impact) + temporal knowledge graph.
- `cairn-embed` — pluggable embeddings (local `fastembed`/ONNX default + OpenAI/Gemini/Voyage/
  Cohere/Ollama); **secret-strip before embed/share.**
- `cairn-mcp` — unified MCP server (`rmcp`), stdio + streamable HTTP, one curated ~26-tool namespace.
- `cairn-api` — `axum` REST + WS; auth (`argon2` accounts, `jsonwebtoken` device tokens), HMAC, TLS.
- `cairn-sync` — offline-first multi-device sync (`automerge` CRDT), optional E2E encryption.
- `cairn-hooks` — hook adapters + `cairn hook <event>` entrypoint.
- `cairn-cli` / `cairn-server` — `cairn` binary: `serve`, `init`, `pair`, `login`,
  `install <agent>` / `install --all` (auto-detect agents), `add-device`, `doctor`, `update`,
  `watch` (TUI, `ratatui`), `sync`, `share`/`pull`, `verify`, `anchor`.

---

## Two new subsystems (the depth the user asked for)

### Context Assembler (anti-context-rot) — `cairn-context`
Given a query + token budget, build the working set: **retrieve** candidates (memory + code +
profile + collective via hybrid search) → **rank** (RRF + rerank) → **de-dup + diversify** (MMR)
→ **pack** under budget → **order for position** (critical/high-signal at the start *and* end,
support in the middle) with structured headers/tags for navigability. Everything compressed is
`expand`-able. Emits an **assembly report** (what's in, what's dropped, why) for the dashboard.
Directly attacks A (context rot, lost-in-the-middle).

### Active Guardrails — `cairn-guard`
- **Ground-truth verification:** before accepting an agent edit/write, diff it against the
  retained original (blob store); flag unexpected deletions/large rewrites (catch the silent 25%
  corruption). Optionally gate on confirmation.
- **Anchor:** capture the task goal/spec at start; periodically check the session's trajectory
  against it; on divergence, **re-inject** the spec + key decisions (re-anchor).
- **Drift/contradiction:** detect statements that contradict stored decisions/facts (graph +
  embedding signals); surface and resolve.
- **Checkpoints:** snapshot working state at intervals / before risky ops; allow rollback/re-anchor.
- **Reliability score:** per-session signals surfaced in the dashboard.
Techniques: deterministic content-hash diffing (cheap), embedding-similarity drift signals,
rule checks against the knowledge graph, optional LLM-as-judge for semantic verification.
Directly attacks B (drift, corruption, reliability).

---

## Compression/assembly layers (all recoverable)

| Layer | Reference | Crate | Recover via |
|---|---|---|---|
| File reads | lean-ctx modes + cache | `cairn-context` | `expand` (blob store) |
| Shell/tool output | rtk filter/group/dedup | `cairn-shell` | `recover` (tee→blob store) |
| Model responses | caveman style (opt-in) | `cairn-context` | original retained |
| Memory recall | agentmemory tiers + RRF | `cairn-memory`/`search` | full record on expand |
| Working set | Context Assembler | `cairn-context` | `expand` any dropped item |

---

## Unified MCP tools (curated ~26)

- **Context:** `read`, `assemble`, `search`, `tree`, `shell`, `graph`, `impact`, `diff`, `expand`, `recover`.
- **Memory:** `remember`, `recall`, `forget`, `timeline`, `consolidate`, `wakeup` (~400-tok bootstrap).
- **Profile:** `profile_get`, `profile_set`, `prefer`.
- **Guard:** `anchor` (set/recall goal), `verify`, `checkpoint`, `reliability`.
- **Collective:** `share`, `pull`, `knowledge_search`, `vote`.
- **Session/devices/admin:** `session_save/load`, `handoff`, `devices`, `sync_status`, `budget`, `health`, `audit`.

---

## Hooks (agent lifecycle)

`cairn install <agent>` wires each agent to call `cairn hook <event>`:
- `SessionStart` → `wakeup` + `anchor`: inject relevant memory, **profile prefs**, top collective knowledge, and the task goal.
- `UserPromptSubmit` → `assemble` context for the prompt (lean, ordered).
- `PreToolUse(Read/Grep/Shell)` → serve compressed/cached view, not full output.
- `PostToolUse(Edit/Write)` → `verify` vs original (guard); capture observation (dedup + secret-strip); detect preferences/corrections.
- periodic / long-session → drift check + `checkpoint`; re-anchor on divergence.
- `PreCompact` → re-inject anchor + critical facts so compaction can't drop them.
- `SessionEnd`/`SubagentStop` → consolidate; offer to `share` sanitized learnings.

---

## Privacy & sanitization (mandatory for OSS + collective + federation)

- **Default private.** Nothing is shared/federated without explicit opt-in.
- **Sanitization pipeline** before any publish/federate: secret detection + strip (keys, tokens,
  `.env`), PII redaction, path/identifier anonymization, optional manual review with **diff
  preview**, consent gate.
- **Provenance + signing:** shared knowledge packs are signed; recall shows source + trust.
- **Revocation:** `unshare` removes from the pool and propagates revocation to subscribers (best-effort).
- **E2E option** for personal multi-device sync; **no telemetry** by default (opt-in anon stats, like rtk).
- **Federation:** servers subscribe to curated, signed packs under trust/scope/rate policies.
- Ship a **SECURITY.md + threat model** focused on the collective/federation surface.

---

## Web Control Plane (operational UI) — you *do* things here, not just watch

One web app (also serves the landing site) = the single pane of glass to **install, connect,
configure, inspect, edit, approve, share, and debug**. Real-time (WS), keyboard-first (**⌘K
command palette** to search memory/code/sessions/collective and run actions), dark brand theme,
**responsive** (check status + approve flags from your phone).

- **Setup wizard (first run):** create account → pick embedding provider → **Add Device**
  (copy-paste installer + QR/pairing code) → **Connect Agents** (one-click per detected agent) →
  green health check. New user productive in minutes. *(ties directly to Install & Onboarding.)*
- **Devices & Agents (install hub):** every device + which agents are configured on each, live
  connection status, **generate install command / pairing code / QR**, mint/revoke tokens, remove a device.
- **Memory workspace (editable):** search; **create / edit / pin / delete** memories; mark
  important; resolve contradictions; view+edit rationale; promote working→semantic; bulk actions.
- **Profile editor:** view / approve / edit learned preferences + do/don't rules; add rules by
  hand; toggle which are active; correction history.
- **Assembler playground (inspector):** type a query + token budget → see exactly what context
  Cairn would feed (in order), what it drops and **why**, the token count; tune the budget;
  `expand` any item. Debug what the agent actually sees — genuinely useful day to day.
- **Reliability center:** review drift events, verification flags, **silent-corruption catches**;
  **approve / reject** flagged edits; **roll back to a checkpoint**; set/adjust the task anchor.
- **Collective / Federation manager:** browse/search the pool; **publish** with sanitization
  **diff preview** + consent; **pull** packs; subscribe to federated servers; manage trust; vote;
  unshare/revoke.
- **Savings & recover:** tokens/$ saved, signed savings ledger, and **expand/recover any
  compressed artifact** (proves no-loss) — for trust + debugging.
- **Sessions:** live stream + replay; jump from a session to its memories/decisions; **Resume
  task** (re-inject the anchor + assembled context into a fresh session on any device).
- **Settings:** embedding provider + keys, budgets/SLOs, roles, privacy/sanitization rules, auth,
  backup / export / import.
- **Overview:** tokens/$ saved, recall/cache hit-rate, reliability score, "smartness lift", active
  devices, recent activity — every tile links straight into the actionable views above.
- *(Stretch)* **Playground chat:** a minimal in-browser chat that runs against your assembled
  context + memory, to demo the "smartness lift" without an external agent.

---

## Install & Onboarding (dead-simple — the user's priority)

**Goal:** install on any device in **one command**, and connect every agent automatically. A
single static Rust binary — **no Node/Python/runtime** to install.

**1. Server (once — home server / NAS / Pi / VPS):**
- One-liner: `curl -fsSL https://cairn.sh/install.sh | sh` · Windows: `irm https://cairn.sh/install.ps1 | iex`.
- Or Docker: `docker run -p 7777:7777 -v cairn:/data ghcr.io/cairn/cairn` · or `docker compose up`.
- Or one-click: Fly / Railway / Render deploy buttons.
- `cairn serve` starts the server **+ embedded web UI**, and prints the URL + a first-run admin link.

**2. Each device (the "easy on every device" part) — Tailscale / `gh`-style pairing:**
- In the web UI, click **Add Device** → it shows a copy-paste one-liner with a short-lived
  pairing code (and a **QR code** for mobile), e.g.:
  `curl -fsSL https://cairn.sh/i | sh -s -- pair CAIRN-7Q3X`
- That command **installs the binary, pairs the device** to your server (device-code flow — no
  manual token juggling), then runs **`cairn install --all`** to **auto-detect installed agents**
  (Claude Code, Codex, OpenCode, Cursor, Windsurf, Cline, Gemini CLI, Copilot…) and write their
  **hook + MCP config** to point at your server.
- Manual paths exist too: `cairn login <server-url>`, `cairn install <agent>`, `cairn pair <code>`,
  `cairn doctor` (verifies hooks + MCP + connectivity).

**3. Connectivity (self-host reality):** default LAN; for remote devices, recommend
**Tailscale/VPN** (zero-config, private) or an optional TLS reverse proxy; an optional relay can
come later. The web UI detects the situation and shows the right URL/QR per device.

**4. Updates:** `cairn update` self-updates the binary; the server flags when an update is available.

---

## Landing Page

1. **Hero** — *"Make any model smart."* Pain→product; **one-command install**; agent logos.
2. **The problem** — the five failure classes (with the "it's the context, not the IQ" insight).
3. **No context lost** — lossless-by-retention, with a live expand/recover demo.
4. **Less, not more** — the Context Assembler vs context rot.
5. **Stay reliable** — guardrails: verify vs originals, drift detection, re-anchor (the 25% catch).
6. **Five pillars** — Remember · Compress · Assemble · Reliable · Smarter together.
7. **Collective knowledge** — opt-in, sanitized, federated; "every traveler adds a stone."
8. **Proof** — benchmark targets (LongMemEval/LoCoMo, token %, recover fidelity, horizon reliability).
9. **Self-host / OSS — install in one command** — `curl … | sh`, single Rust binary, "runs on a
   Pi"; **Add-Device pairing** (copy-paste + QR) and `cairn install --all` auto-configures every
   detected agent; Docker + one-click templates; federation.
10. **Privacy/security** + **CTA** (install + docs + GitHub).

---

## Open-source & community

- **License:** **Apache-2.0** for the core (permissive, max adoption, matches rtk). *(Alt: AGPL
  if you later want to stop closed SaaS forks of a hosted collective.)*
- **Repo:** monorepo — Cargo workspace + `/web` (Next.js) + `/docs`.
- **Install:** one-command shell installer + Homebrew/cargo + prebuilt binaries (musl, mac
  arm/x86, windows); `cairn install <agent>` auto-configs 15+ agents (hooks + MCP).
- **Project files:** README, CONTRIBUTING, SECURITY.md + threat model, governance, Discord/community.
- **CI:** `cargo test`/`clippy`/`fmt`, web build, docker build, multi-platform release, **benchmark CI**.

---

## Tech Stack

- **Engine (Rust):** `tokio`, `axum`, `sqlx` (SQLite+Postgres), `sqlite-vec`/`pgvector`,
  `tantivy`, `tree-sitter`(+grammars), `fastembed`, `rmcp` (MCP), `ratatui`, `automerge`,
  `argon2`+`jsonwebtoken`, `rust-embed`.
- **UI (one Next.js app: landing + dashboard):** Next.js App Router + Tailwind + shadcn/ui,
  Recharts, Cytoscape/react-force-graph; talks to `cairn-api`; built static + embedded in binary.
- **Packaging:** musl static binary, distroless Docker, `docker compose` (+ Postgres/pgvector),
  Fly/Railway/Render templates.

---

## Build Phases (parallel tracks — each ships a thin slice first)

**Phase 0 — Scaffold:** monorepo (Cargo workspace + Next.js + docker compose) + CI; brand tokens
(name/logo/palette/fonts) → Tailwind theme; README + license.

**Phase 1 — Thin vertical slice, both tracks:**
- *Engine:* `cairn-context` (read modes + cache + **Assembler** + **expand**) + `cairn-shell`
  (compress + **recover**) + `cairn-memory`/`search` minimal (`remember`/`recall`/`wakeup`) →
  via `cairn-mcp` (stdio) + `cairn-api`. Prove: re-read killer, lean assembly, byte-identical recover.
- *Product:* brand identity → landing live → **web control plane** shell: Setup wizard +
  **Add-Device / Connect-Agent install hub**, Assembler playground, and Context/Memory views with
  a working expand/recover demo, wired to `cairn-api` (sample data until endpoints land).

**Phase 2 — Server, sync, smart + guard:** auth + device tokens; `cairn-sync` reconcile;
`cairn-profile` (preference learning); **`cairn-guard`** (verify vs original, drift, re-anchor,
checkpoints); Docker server; **one-line installer + device-code pairing + `cairn install --all`
auto-detect + `cairn doctor`** + hooks bundle. Verify 2 devices share memory+profile; verify a
corrupted edit is flagged.

**Phase 3 — Collective + federation + depth:** `cairn-collective` (sanitize→consent→share→pull→
vote), signed packs + federation; full 4-tier consolidation/decay; property graph + impact;
hybrid rerank; budgets/SLOs; **benchmarks (LongMemEval/LoCoMo/token/horizon)**; docs; one-click deploy.

---

## Verification

- **No-loss (headline):** compress a 1000-line file + a 200-line failing-test output → window
  cost −70–90% → `expand`/`recover` returns the **byte-identical** original.
- **Anti-rot:** `assemble` under a tight budget puts the goal + key decisions at the edges; show
  the assembly report; confirm dropped items are `expand`-able.
- **Re-read killer:** unchanged file re-read → ~13 tokens; edit then re-read → diff-only.
- **Reliability/guard:** make an agent edit that deletes unrelated content → `verify` flags it;
  drift from the task goal → re-anchor fires; checkpoint rollback works.
- **Memory + smart:** fact saved in session A recalled in fresh session B; with a small model,
  profile + collective injection changes output to honor learned preferences vs. baseline.
- **Collective + privacy:** `share` → sanitization strips secrets/PII (diff preview) → another
  account `pull`s with provenance → `unshare` revokes; federation pulls only signed packs.
- **Multi-device:** memory/profile on one container recalled on another after sync; revoke token cuts access.
- **Install/onboarding:** on a clean machine, the **Add-Device one-liner** installs the binary,
  pairs to the server (device-code), and `cairn install --all` configures a detected agent;
  `cairn doctor` is green; that agent's next session hits Cairn (wakeup fires).
- **UI/packaging/bench:** dashboard click-through on live server; Lighthouse on landing;
  `docker compose up` from clean checkout; benchmark CI emits LongMemEval/LoCoMo/token numbers.

---

## Scope & risks

- **Scope discipline:** multi-month product. Parallel tracks, but each ships a **thin** slice
  before depth. **Multi-agent orchestration is out of v1** (single-agent reliability first — the
  research shows single-agent often wins and coordination is where multi-agent fails).
- **Privacy of collective/federation is the #1 risk** — sanitization + consent + provenance +
  revocation must be airtight before any public pool/federation ships. Everything defaults private.
- **Lossy response-style compression (caveman) stays opt-in** and never touches stored fidelity.
- **Rust maturity:** confirm `rmcp` + `fastembed`; `automerge` sync + federation are the most
  novel pieces — prototype early (Phase 2/3).
- **Crowded memory market:** differentiate via the *integration* (memory + no-loss + assembly +
  guardrails + federation) and publish honest benchmarks; don't claim numbers until measured.
- **Naming/domain:** finalize Cairn vs. backups and secure a domain + the GitHub org before launch copy.
