"use client";

import { useEffect, useState } from "react";
import { HelpCircle, X } from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

const STORAGE_KEY = "cairn-infocard-dismissed-v1";

export interface InfoCardProps {
  what: string;
  how: string[];
  impact: string;
  className?: string;
}

export function InfoCard({ what, how, impact, className }: InfoCardProps) {
  const [dismissed, setDismissed] = useState(false);
  const [hydrated, setHydrated] = useState(false);

  useEffect(() => {
    setHydrated(true);
    try {
      const raw = window.sessionStorage.getItem(STORAGE_KEY);
      const set = raw ? (JSON.parse(raw) as Record<string, boolean>) : {};
      if (set[currentKey(what)]) {
        setDismissed(true);
      }
    } catch {
      /* ignore */
    }
  }, [what]);

  const handleDismiss = () => {
    setDismissed(true);
    try {
      const raw = window.sessionStorage.getItem(STORAGE_KEY);
      const set = raw ? (JSON.parse(raw) as Record<string, boolean>) : {};
      set[currentKey(what)] = true;
      window.sessionStorage.setItem(STORAGE_KEY, JSON.stringify(set));
    } catch {
      /* ignore */
    }
  };

  const handleReopen = () => setDismissed(false);

  if (!hydrated) {
    return null;
  }

  if (dismissed) {
    return (
      <div className={cn("flex justify-end", className)}>
        <Button
          type="button"
          variant="ghost"
          size="icon"
          onClick={handleReopen}
          aria-label="Show help for this section"
          title="Show help for this section"
          className="size-7 text-muted-foreground"
        >
          <HelpCircle className="size-4" aria-hidden="true" />
        </Button>
      </div>
    );
  }

  return (
    <Card
      className={cn(
        "border-l-2 border-l-[hsl(var(--color-info))] bg-[hsl(var(--color-info))]/5",
        className,
      )}
      role="region"
      aria-label="Section help"
    >
      <CardContent className="p-4">
        <div className="flex items-start justify-between gap-3">
          <div className="flex-1 space-y-2 text-sm">
            <div>
              <span className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                What this is
              </span>
              <p className="mt-0.5 text-foreground/90">{what}</p>
            </div>
            <div>
              <span className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                How to use it
              </span>
              <ul className="mt-0.5 list-disc space-y-0.5 pl-5 text-foreground/90">
                {how.map((line, i) => (
                  <li key={i}>{line}</li>
                ))}
              </ul>
            </div>
            <div>
              <span className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                Impact on Cairn
              </span>
              <p className="mt-0.5 text-foreground/90">{impact}</p>
            </div>
          </div>
          <Button
            type="button"
            variant="ghost"
            size="icon"
            onClick={handleDismiss}
            aria-label="Dismiss this help"
            title="Dismiss this help"
            className="size-7 shrink-0 text-muted-foreground"
          >
            <X className="size-4" aria-hidden="true" />
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}

function currentKey(what: string): string {
  return what.slice(0, 40);
}
