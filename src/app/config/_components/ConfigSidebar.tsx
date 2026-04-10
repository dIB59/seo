"use client";
import pkg from "@/package.json";

import { useRouter } from "next/navigation";
import { LayoutDashboard, Bot, CreditCard, Palette, ChevronLeft, Puzzle, Code2, FileBarChart2, Tags } from "lucide-react";
import { Button } from "@/src/components/ui/button";

export const SIDEBAR_ITEMS = [
  { id: "report-builder", label: "Report Builder", icon: FileBarChart2 },
  { id: "ai", label: "AI Settings", icon: Bot },
  { id: "prompt", label: "Analysis Prompt", icon: LayoutDashboard },
  { id: "custom-checks", label: "Custom Checks", icon: Puzzle },
  { id: "custom-extractors", label: "Custom Extractors", icon: Code2 },
  { id: "tags", label: "Tags", icon: Tags },
  { id: "licensing", label: "Licensing", icon: CreditCard },
  { id: "appearance", label: "Appearance", icon: Palette },
];

interface ConfigSidebarProps {
  activeSection: string;
  setActiveSection: (section: string) => void;
}

export function ConfigSidebar({ activeSection, setActiveSection }: ConfigSidebarProps) {
  const router = useRouter();

  return (
    <aside className="w-64 border-r border-border/40 bg-card/30 flex flex-col relative overflow-hidden backdrop-blur-sm">
      {/* Subtle noise/gradient texture */}
      <div className="absolute inset-0 bg-gradient-to-b from-transparent via-primary/5 to-transparent opacity-50 pointer-events-none" />

      <div className="p-4 border-b border-border/40 flex items-center gap-2 relative z-10">
        <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => router.push("/")}>
          <ChevronLeft className="h-4 w-4" />
        </Button>
        <span className="font-semibold tracking-tight">Configuration</span>
      </div>
      <nav className="flex-1 p-2 space-y-1 relative z-10">
        {SIDEBAR_ITEMS.map((item) => (
          <button
            key={item.id}
            onClick={() => setActiveSection(item.id)}
            className={`w-full flex items-center gap-3 px-3 py-2 text-sm font-medium rounded-md transition-all duration-200 group relative overflow-hidden ${
              activeSection === item.id
                ? "bg-muted/80 text-foreground font-semibold shadow-sm border border-border/50"
                : "text-muted-foreground hover:bg-muted/50 hover:text-foreground"
            }`}
          >
            <item.icon
              className={`h-4 w-4 transition-transform duration-300 ${activeSection === item.id ? "text-primary" : "group-hover:text-primary"}`}
            />
            {item.label}
            {activeSection === item.id && (
              <div className="absolute left-0 top-1/2 -translate-y-1/2 w-0.5 h-1/2 bg-primary rounded-r-full" />
            )}
          </button>
        ))}
      </nav>
      <div className="p-4 border-t border-border/40 text-xs text-muted-foreground relative z-10 flex justify-between items-center">
        <span>SEO Insikt</span>
        <span className="font-mono opacity-50">v{pkg.version}</span>
      </div>
    </aside>
  );
}
