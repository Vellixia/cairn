"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useEffect, useState } from "react";
import {
  LayoutDashboard,
  Brain,
  ShieldCheck,
  UserCircle,
  type LucideIcon,
} from "lucide-react";
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar";
import { cn } from "@/lib/utils";
import Logo from "@/components/Logo";

type Item = { href: string; label: string; icon: LucideIcon };

const ITEMS: Item[] = [
  { href: "/dashboard", label: "Now", icon: LayoutDashboard },
  {
    href: "/dashboard?view=memory",
    label: "Memory & Context",
    icon: Brain,
  },
  { href: "/dashboard?view=trust", label: "Trust", icon: ShieldCheck },
  { href: "/dashboard?view=you", label: "You", icon: UserCircle },
];

const STORAGE_KEY = "cairn-sidebar-v2";

function isActive(pathname: string | null, href: string): boolean {
  if (!pathname) return false;
  if (href === "/dashboard") {
    return pathname === "/dashboard" || pathname === "/dashboard/";
  }
  // query-aware match for hub links
  const [path, query] = href.split("?");
  if (pathname !== path) return false;
  if (!query) return true;
  if (typeof window === "undefined") return false;
  const params = new URLSearchParams(window.location.search);
  const wanted = new URLSearchParams(query);
  for (const [k, v] of wanted) {
    if (params.get(k) !== v) return false;
  }
  return true;
}

function NavLink({
  item,
  pathname,
  active,
}: {
  item: Item;
  pathname: string | null;
  active: boolean;
}) {
  const Icon = item.icon;
  return (
    <SidebarMenuButton asChild isActive={active} size="sm">
      <Link
        href={item.href}
        aria-current={active ? "page" : undefined}
        onClick={() => {
          if (typeof window === "undefined") return;
          try {
            window.localStorage.setItem(STORAGE_KEY, "1");
          } catch {
            /* ignore */
          }
        }}
      >
        <Icon />
        <span>{item.label}</span>
      </Link>
    </SidebarMenuButton>
  );
}

export function CairnSidebar() {
  const pathname = usePathname();
  const [hydrated, setHydrated] = useState(false);

  useEffect(() => {
    // Migration: v1 (8-group) -> v2 (4-entry). Drop unknown ids; no group-open state preserved.
    try {
      const v1 = window.localStorage.getItem("cairn-sidebar-v1");
      if (v1) {
        window.localStorage.removeItem("cairn-sidebar-v1");
      }
    } catch {
      /* ignore */
    }
    try {
      window.localStorage.setItem(STORAGE_KEY, "1");
    } catch {
      /* ignore */
    }
    setHydrated(true);
  }, []);

  return (
    <Sidebar collapsible="offcanvas" className="border-r border-line">
      <SidebarHeader className="border-b border-line">
        <div className="flex items-center gap-2 px-2 py-2">
          <Logo size={26} />
          <span className="font-semibold tracking-tight">Cairn</span>
        </div>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel
            className={cn(
              "flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-xs font-semibold uppercase tracking-wide text-muted-foreground",
            )}
          >
            <span>Now</span>
          </SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {hydrated
                ? ITEMS.map((it) => (
                    <SidebarMenuItem key={it.href}>
                      <NavLink
                        item={it}
                        pathname={pathname}
                        active={isActive(pathname, it.href)}
                      />
                    </SidebarMenuItem>
                  ))
                : null}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
    </Sidebar>
  );
}
