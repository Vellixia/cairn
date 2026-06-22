"use client";

import { Suspense } from "react";
import { useSearchParams } from "next/navigation";
import { OverviewContent } from "./OverviewContent";
import { HubTabs, type HubTab } from "@/components/HubTabs";
import MemoryPage from "./memory/page";
import RecallPage from "./memory/recall/page";
import WakeupPage from "./memory/wakeup/page";
import MemoryGraphPage from "./memory/graph/page";
import ContextInspectorPage from "./context/page";
import AssemblePage from "./context/assemble/page";
import SavingsPage from "./savings/page";
import ReliabilityScorePage from "./reliability/page";
import AnchorPage from "./reliability/anchor/page";
import CheckpointsPage from "./reliability/checkpoints/page";
import DriftCenterPage from "./reliability/drift/page";
import SanitizePage from "./share/sanitize/page";
import BundlePage from "./share/export/page";
import PoolPage from "./pool/page";
import RegistryPage from "./registry/page";
import ProfilePage from "./profile/page";
import DevicesTokensPage from "./devices/page";
import PairCodePage from "./devices/pair/page";
import AuditPage from "./devices/audit/page";
import SessionsPage from "./sessions/page";
import SettingsPage from "./settings/page";

const MEMORY_TABS: HubTab[] = [
  { id: "remember", label: "Remember", content: <MemoryPage /> },
  { id: "recall", label: "Recall", content: <RecallPage /> },
  { id: "wakeup", label: "Wakeup", content: <WakeupPage /> },
  { id: "graph", label: "Graph", content: <MemoryGraphPage /> },
  { id: "inspector", label: "Inspector", content: <ContextInspectorPage /> },
  { id: "assemble", label: "Assemble", content: <AssemblePage /> },
  { id: "savings", label: "Savings", content: <SavingsPage /> },
];

const TRUST_TABS: HubTab[] = [
  { id: "score", label: "Score", content: <ReliabilityScorePage /> },
  { id: "anchor", label: "Anchor", content: <AnchorPage /> },
  { id: "checkpoints", label: "Checkpoints", content: <CheckpointsPage /> },
  { id: "drift", label: "Drift", content: <DriftCenterPage /> },
  { id: "sanitize", label: "Sanitize", content: <SanitizePage /> },
  { id: "bundles", label: "Bundles", content: <BundlePage /> },
  { id: "pool", label: "Pool", content: <PoolPage /> },
  { id: "registry", label: "Registry", content: <RegistryPage /> },
];

const YOU_TABS: HubTab[] = [
  { id: "profile", label: "Profile", content: <ProfilePage /> },
  { id: "tokens", label: "Tokens", content: <DevicesTokensPage /> },
  { id: "pair", label: "Pair", content: <PairCodePage /> },
  { id: "audit", label: "Audit", content: <AuditPage /> },
  { id: "sessions", label: "Sessions", content: <SessionsPage /> },
  { id: "settings", label: "Settings", content: <SettingsPage /> },
];

function DashboardPageInner() {
  const params = useSearchParams();
  const view = params.get("view");

  if (view === "memory") {
    return (
      <HubTabs
        view="memory"
        title="Memory & Context"
        description="Write, recall, and read — the core loop. Plus the byte-savings ledger that proves it works."
        tabs={MEMORY_TABS}
        defaultTab="remember"
      />
    );
  }
  if (view === "trust") {
    return (
      <HubTabs
        view="trust"
        title="Trust"
        description="Reliability, the drift center, and the way memories leave this Cairn."
        tabs={TRUST_TABS}
        defaultTab="score"
      />
    );
  }
  if (view === "you") {
    return (
      <HubTabs
        view="you"
        title="You"
        description="Your profile, your devices, your sessions."
        tabs={YOU_TABS}
        defaultTab="profile"
      />
    );
  }

  return (
    <div className="space-y-6">
      <header className="space-y-1">
        <h1 className="text-2xl font-semibold tracking-tight">Now</h1>
        <p className="text-sm text-muted-foreground">
          Server health, reliability, recent memory, and the last few admin actions — at a glance.
        </p>
      </header>
      <OverviewContent />
    </div>
  );
}

export default function DashboardOverviewPage() {
  return (
    <Suspense fallback={null}>
      <DashboardPageInner />
    </Suspense>
  );
}
