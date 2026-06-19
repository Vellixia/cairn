"use client";

import { useEffect, useState } from "react";

const SHORTCUTS: { keys: string; desc: string }[] = [
  { keys: "⌘K / Ctrl+K", desc: "Toggle the command palette" },
  { keys: "?", desc: "Toggle this shortcuts modal" },
  { keys: "esc", desc: "Close any open dialog" },
];

export function Shortcuts() {
  const [open, setOpen] = useState(false);

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      // ? opens the shortcuts modal, but only when not typing in an input
      if (e.key === "?" && !isTyping(e.target)) {
        e.preventDefault();
        setOpen((v) => !v);
      } else if (e.key === "Escape" && open) {
        setOpen(false);
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open]);

  if (!open) return null;
  return (
    <div
      role="dialog"
      aria-label="Keyboard shortcuts"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4"
      onClick={() => setOpen(false)}
    >
      <div
        className="w-full max-w-md rounded-xl border border-line bg-surface p-6 shadow-2xl shadow-black/50"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between">
          <h2 className="text-base font-semibold">Keyboard shortcuts</h2>
          <button
            type="button"
            onClick={() => setOpen(false)}
            className="rounded-md border border-line px-2 py-0.5 text-xs hover:bg-surface2"
            aria-label="Close"
          >
            esc
          </button>
        </div>
        <ul className="mt-4 space-y-2">
          {SHORTCUTS.map((s) => (
            <li key={s.keys} className="flex items-baseline justify-between gap-3 text-sm">
              <span className="text-slate">{s.desc}</span>
              <kbd className="font-mono rounded border border-line bg-surface2 px-1.5 py-0.5 text-xs">{s.keys}</kbd>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}

function isTyping(target: EventTarget | null): boolean {
  const el = target as HTMLElement | null;
  if (!el) return false;
  const tag = el.tagName;
  return (
    tag === "INPUT" ||
    tag === "TEXTAREA" ||
    tag === "SELECT" ||
    el.isContentEditable
  );
}
