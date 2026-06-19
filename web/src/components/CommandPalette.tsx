"use client";

import { Command } from "cmdk";
import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";

interface Item {
  id: string;
  label: string;
  hint?: string;
  group: "Navigate" | "Memory" | "Reliability" | "Devices" | "Share";
  action: () => void;
}

export function CommandPalette() {
  const router = useRouter();
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      // ⌘K / Ctrl+K — toggle
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setOpen((v) => !v);
      } else if (e.key === "Escape" && open) {
        setOpen(false);
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open]);

  const nav = (href: string) => () => {
    setOpen(false);
    router.push(href);
  };

  const items: Item[] = [
    { id: "nav-overview", label: "Overview", group: "Navigate", action: nav("/dashboard") },
    { id: "nav-settings", label: "Settings", group: "Navigate", action: nav("/dashboard/settings") },
    { id: "nav-mem", label: "Memories · Remember", group: "Navigate", action: nav("/dashboard/memory") },
    { id: "nav-recall", label: "Memories · Recall", group: "Navigate", action: nav("/dashboard/memory/recall") },
    { id: "nav-wakeup", label: "Memories · Wakeup", group: "Navigate", action: nav("/dashboard/memory/wakeup") },
    { id: "nav-ctx", label: "Context · Inspector", group: "Navigate", action: nav("/dashboard/context") },
    { id: "nav-asm", label: "Context · Assemble", group: "Navigate", action: nav("/dashboard/context/assemble") },
    { id: "nav-rel", label: "Reliability · Score", group: "Navigate", action: nav("/dashboard/reliability") },
    { id: "nav-anchor", label: "Reliability · Anchor", group: "Navigate", action: nav("/dashboard/reliability/anchor") },
    { id: "nav-cp", label: "Reliability · Checkpoints", group: "Navigate", action: nav("/dashboard/reliability/checkpoints") },
    { id: "nav-san", label: "Share · Sanitize", group: "Navigate", action: nav("/dashboard/share/sanitize") },
    { id: "nav-bun", label: "Share · Bundles", group: "Navigate", action: nav("/dashboard/share/export") },
    { id: "nav-pool", label: "Pool", group: "Navigate", action: nav("/dashboard/pool") },
    { id: "nav-devs", label: "Devices · Tokens", group: "Navigate", action: nav("/dashboard/devices") },
    { id: "nav-pair", label: "Devices · Pair new", group: "Navigate", action: nav("/dashboard/devices/pair") },
    { id: "nav-audit", label: "Devices · Audit", group: "Navigate", action: nav("/dashboard/devices/audit") },
    // Memory shortcuts
    { id: "act-remember", label: "Remember something", hint: "jump to Memories", group: "Memory", action: nav("/dashboard/memory") },
    { id: "act-recall", label: "Recall a memory", hint: "jump to Recall", group: "Memory", action: nav("/dashboard/memory/recall") },
    // Reliability
    { id: "act-cp", label: "Create a checkpoint", hint: "jump to Checkpoints", group: "Reliability", action: nav("/dashboard/reliability/checkpoints") },
    // Devices
    { id: "act-issue", label: "Issue a device token", hint: "jump to Tokens", group: "Devices", action: nav("/dashboard/devices") },
    // Share
    { id: "act-san", label: "Sanitize text", hint: "jump to Sanitize", group: "Share", action: nav("/dashboard/share/sanitize") },
  ];

  return (
    <Command.Dialog
      open={open}
      onOpenChange={setOpen}
      label="Cairn command palette"
      className="cairn-dialog fixed left-1/2 top-24 z-50 w-[min(640px,92vw)] -translate-x-1/2 rounded-xl border border-line bg-surface shadow-2xl shadow-black/50"
    >
      <div className="border-b border-line">
        <Command.Input
          autoFocus
          value={query}
          onValueChange={setQuery}
          placeholder="Jump to a section, run an action…"
          className="w-full bg-transparent px-4 py-3 text-sm outline-none placeholder:text-slate"
        />
      </div>
      <Command.List className="max-h-[60vh] overflow-y-auto p-2">
        <Command.Empty className="px-3 py-6 text-center text-sm text-slate">
          No matches. Try a section name like "memory" or "tokens".
        </Command.Empty>
        {(["Navigate", "Memory", "Reliability", "Devices", "Share"] as const).map((group) => {
          const filtered = items.filter((i) => i.group === group);
          if (filtered.length === 0) return null;
          return (
            <Command.Group key={group} heading={group} className="[&_[cmdk-group-heading]]:px-2 [&_[cmdk-group-heading]]:py-1 [&_[cmdk-group-heading]]:text-[10.5px] [&_[cmdk-group-heading]]:uppercase [&_[cmdk-group-heading]]:tracking-wider [&_[cmdk-group-heading]]:text-slate">
              {filtered.map((it) => (
                <Command.Item
                  key={it.id}
                  value={`${it.label} ${it.hint ?? ""}`}
                  onSelect={it.action}
                  className="flex cursor-pointer items-center gap-3 rounded-md px-3 py-2 text-sm aria-selected:bg-surface2 aria-selected:text-offwhite data-[selected=true]:bg-surface2 data-[selected=true]:text-offwhite"
                >
                  <span className="flex-1 truncate">{it.label}</span>
                  {it.hint && <span className="text-[11px] text-slate truncate">{it.hint}</span>}
                </Command.Item>
              ))}
            </Command.Group>
          );
        })}
      </Command.List>
      <div className="flex items-center justify-between border-t border-line px-3 py-2 text-[11px] text-slate">
        <span><kbd className="font-mono">↑↓</kbd> navigate · <kbd className="font-mono">↵</kbd> select · <kbd className="font-mono">esc</kbd> close</span>
        <span><kbd className="font-mono">?</kbd> for shortcuts</span>
      </div>
    </Command.Dialog>
  );
}
