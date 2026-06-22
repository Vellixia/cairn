import type { InfoCardProps } from "@/components/InfoCard";

export type InfoCopy = Omit<InfoCardProps, "className">;

export const INFOCOPY: Record<string, InfoCopy> = {
  "/dashboard/memory": {
    what: "Write durable memories Cairn can recall later in this session or any future one.",
    how: [
      "Type a fact, paste a snippet, or capture from the browser extension.",
      "Pick a tier (working / session / long) so Cairn knows how long to keep it.",
      "Press Enter to save. Cairn embeds + indexes it for recall immediately.",
    ],
    impact:
      "Each memory costs a small embed + write. Long-tier memories shape future recall; working-tier ones are dropped automatically.",
  },
  "/dashboard/memory/recall": {
    what: "Search across every memory Cairn has, ranked by BM25 + semantic similarity.",
    how: [
      "Type a question or phrase. Empty query returns your most-recent items.",
      "Click a result to expand it. Pin a hit to keep it in the current context.",
    ],
    impact:
      "Recall runs against the local Helix index in <50ms for 10k items. Pinning extends a memory's working-tier TTL by 30 minutes.",
  },
  "/dashboard/memory/wakeup": {
    what: "The high-importance memories Cairn would surface at the start of a new session.",
    how: [
      "Browse the ranked list. Items at the top are the most likely to be relevant right now.",
      "Click an item to read it in full or to remove it from wakeup.",
    ],
    impact:
      "Wakeup is the default context Cairn loads for a fresh agent. Trimming here directly shrinks every future session's token bill.",
  },
  "/dashboard/memory/graph": {
    what: "A live map of relationships between memories, extracted from their edges.",
    how: [
      "Drag to pan, scroll to zoom. Click a node to focus it and its neighbours.",
      "Use the search box to jump to a specific memory.",
    ],
    impact:
      "Edges are auto-derived from co-recall and explicit links. Graph traversal is what makes proactive_recall fire on related cues.",
  },
  "/dashboard/context": {
    what: "Read files in your project with Cairn's AST-aware modes — cheaper than raw text.",
    how: [
      "Type a path or browse the tree. Pick a mode: full, outline, signatures, map.",
      "Cheaper modes collapse to structure only and feed less into your context window.",
    ],
    impact:
      "Outline mode cuts code reads by ~90% (59k tok → 5.9k tok in our fixtures). Use it first; fall back to full only when you need exact lines.",
  },
  "/dashboard/context/assemble": {
    what: "Pack a question + the right memories + a code slice into a token-budgeted prompt.",
    how: [
      "Type your question, pick a budget, then click Assemble.",
      "Review the diff view to see what Cairn picked and why. Tune by adding or pinning memories.",
    ],
    impact:
      "Assembly enforces a hard token ceiling so you never blow the model's context. The diff view lets you audit the cut, memory by memory.",
  },
  "/dashboard/savings": {
    what: "The tamper-evident ledger of every byte Cairn has saved you by reading less.",
    how: [
      "Filter by date or source. The total at the top is your running saved-bytes counter.",
      "Click 'Verify' to re-check the chain. Any mismatch means the ledger was tampered with.",
    ],
    impact:
      "Bytes saved → tokens saved → USD saved. This page is the proof that the read modes are actually doing their job.",
  },
  "/dashboard/reliability": {
    what: "Cairn's edit-guard score: how often your edits round-trip cleanly through memory.",
    how: [
      "Watch the score trend. Each sample is one edit + one re-read.",
      "Click a sample to see what changed and whether Cairn flagged it.",
    ],
    impact:
      "Score < 70 means drift is likely. Rollback from the Checkpoints tab before the bad state spreads across devices.",
  },
  "/dashboard/reliability/anchor": {
    what: "The current task anchor — a one-line summary that scopes every recall to 'what you are doing right now'.",
    how: [
      "Type the shortest sentence that names your current goal. Save.",
      "Update it whenever the task changes; Cairn re-ranks recall around it automatically.",
    ],
    impact:
      "Anchored recall returns 2-3x more relevant hits and uses ~40% fewer tokens per query. No anchor = no scoping = generic recall.",
  },
  "/dashboard/reliability/checkpoints": {
    what: "Named snapshots of the memory + reliability state you can roll back to.",
    how: [
      "Click 'Create checkpoint' before any risky edit. Cairn stores the current state.",
      "Click 'Rollback' on any checkpoint to restore it. Cairn records the rollback in the audit log.",
    ],
    impact:
      "Checkpoints are the only safe way to test changes that touch many memories. Always checkpoint before editing >5 memories at once.",
  },
  "/dashboard/reliability/drift": {
    what: "The drift center — every reliability sample flagged as ok, warn, or danger.",
    how: [
      "Filter by status. Click any sample to see the diff and the AI's reasoning.",
      "If a danger sample is wrong, mark it resolved. Cairn adjusts the score.",
    ],
    impact:
      "Drift is the leading indicator of reliability decay. Check this page weekly if you have heavy editing traffic.",
  },
  "/dashboard/share/sanitize": {
    what: "Redact secrets, PII, and sensitive paths from any text before it leaves your machine.",
    how: [
      "Paste text or a file path. Cairn scans for keys, emails, and project-relative paths.",
      "Classify each finding as shareable / needs_review / private. Sanitize, then copy or export.",
    ],
    impact:
      "Sanitize runs locally — nothing is uploaded. The sensitivity tier is preserved in the bundle so downstream readers know what to handle carefully.",
  },
  "/dashboard/share/export": {
    what: "Bundle a sanitized subset of memories into a portable .cairnpkg for sharing with another Cairn instance.",
    how: [
      "Pick a sensitivity ceiling (shareable / needs_review) and an anchor (optional).",
      "Click Build. Cairn signs the bundle with your key and stores the hash in the ledger.",
    ],
    impact:
      "Exports are signed, so the recipient can verify nothing was tampered with on the wire. Imports always check the signature.",
  },
  "/dashboard/pool": {
    what: "Federate with other Cairn instances you trust. Memories cross-pollinate by anchor.",
    how: [
      "Add a trusted key + URL. Cairn pings and lists their public packs.",
      "Pull a pack to mirror their memories into your recall pool under your anchor.",
    ],
    impact:
      "Pools only share memories scoped to the same anchor. No key = no access. Pool traffic is end-to-end encrypted.",
  },
  "/dashboard/registry": {
    what: "Browse signed memory packs published by the Cairn community.",
    how: [
      "Search by name or tag. Click a pack to see its manifest, signature, and reviews.",
      "Install with one click. Cairn verifies the Ed25519 signature before adding the pack.",
    ],
    impact:
      "Registry packs are signed by their author; Cairn refuses to install unsigned or revoked packs. Trust flows from the cairn-registry keyring.",
  },
  "/dashboard/profile": {
    what: "Your personal preferences, sensitivity defaults, and sharing settings.",
    how: [
      "Edit your profile (display name, default sensitivity). Save.",
      "Use the toggles to set default sharing tier and whether Cairn logs in dev mode.",
    ],
    impact:
      "Defaults here cascade into every sanitize / export action. Setting 'shareable' as default makes builds faster; 'private' is safer.",
  },
  "/dashboard/devices": {
    what: "Issue, list, and revoke the tokens your CLI / MCP clients use to talk to this server.",
    how: [
      "Click 'Issue token'. Pick a name and scope (admin / write / read).",
      "Copy the token from the response — it is shown once. Revoke here when a device is lost.",
    ],
    impact:
      "Tokens are bearer credentials. Revoke immediately on loss; expired tokens are rejected, not auto-rotated.",
  },
  "/dashboard/devices/pair": {
    what: "Generate a short-lived pairing code so a new device can fetch its own token out-of-band.",
    how: [
      "Click 'Generate code'. Read the 6-character code to the new device out-of-band.",
      "The new device hits /api/devices/pair with the code and gets a token.",
    ],
    impact:
      "Pair codes expire in 10 minutes and are single-use. They avoid typing long tokens over an insecure channel.",
  },
  "/dashboard/devices/audit": {
    what: "The last 50 administrative events on this server: logins, token issues, rollbacks, exports.",
    how: [
      "Filter by kind or actor. Each row links to the relevant page.",
      "Audit entries are append-only. The chain is verified nightly.",
    ],
    impact:
      "The audit log is the source of truth for 'who did what when'. It feeds into the savings chart and the activity timeline on the overview.",
  },
  "/dashboard/sessions": {
    what: "The active agent sessions connected to this server.",
    how: [
      "Click a session to see its anchor, current context, and the memories it has loaded.",
      "End a session to drop its working-tier memories and free the slot.",
    ],
    impact:
      "Each session holds working-tier memory. End idle sessions to keep the tier lean and recall fast.",
  },
  "/dashboard/settings": {
    what: "Server info, your admin session, and the sign-out button.",
    how: [
      "View the server version and uptime. Sign out to invalidate this browser's session cookie.",
      "Rotate your admin password from the CLI — invalidates all sessions.",
    ],
    impact:
      "Settings here are minimal because most config lives server-side. Sign out from a shared browser, not just close the tab.",
  },
};
