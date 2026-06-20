"use client";

import { SessionGate } from "@/components/SessionGate";
import { CairnSidebar } from "@/components/Sidebar";
import { Topbar } from "@/components/Topbar";
import { CommandPalette } from "@/components/CommandPalette";
import { Shortcuts } from "@/components/Shortcuts";
import { SidebarProvider } from "@/components/ui/sidebar";

/**
 * Dashboard shell. Wraps everything in <SessionGate>, which probes auth + redirects unauth'd
 * users to /login. <SidebarProvider> establishes the flex row that holds the sidebar and the
 * main content side-by-side (matches shadcn sidebar pattern). The command palette and
 * shortcuts modal are mounted here so they're available on every dashboard page.
 */
export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <SessionGate>
      <SidebarProvider>
        <CairnSidebar />
        <div className="flex-1 flex min-w-0 flex-col">
          <Topbar />
          <main className="flex-1 px-5 py-6 md:px-8 md:py-8 max-w-[1400px] w-full mx-auto">
            {children}
          </main>
        </div>
      </SidebarProvider>
      <CommandPalette />
      <Shortcuts />
    </SessionGate>
  );
}
