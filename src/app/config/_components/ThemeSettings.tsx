"use client";

import { useTheme } from "next-themes";
import { useSyncExternalStore } from "react";
import { Sun, Moon, Briefcase, Check } from "lucide-react";
import { Label } from "@/src/components/ui/label";

const THEMES = [
  {
    id: "light",
    label: "Light",
    icon: Sun,
    description: "Clean and bright interface",
    colors: {
      bg: "#ffffff",
      card: "#f5f5f5",
      primary: "#1a1a1a",
      accent: "#e5e5e5",
    },
  },
  {
    id: "dark",
    label: "Dark",
    icon: Moon,
    description: "Easy on the eyes",
    colors: {
      bg: "#1a1a1a",
      card: "#262626",
      primary: "#a3a3f5",
      accent: "#333333",
    },
  },
  {
    id: "business",
    label: "Business",
    icon: Briefcase,
    description: "Brand colors on light",
    colors: {
      bg: "#faf9f8",
      card: "#ffffff",
      primary: "#E84D00",
      accent: "#E98C00",
    },
  },
] as const;

export function ThemeSettings() {
  const { theme, setTheme } = useTheme();
  const mounted = useSyncExternalStore(
    () => () => {},
    () => true,
    () => false,
  );

  if (!mounted) {
    return (
      <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
        <div className="space-y-2">
          <Label className="text-base">Theme</Label>
          <p className="text-sm text-muted-foreground">Choose your preferred appearance.</p>
        </div>
        <div className="grid grid-cols-3 gap-4">
          {[1, 2, 3].map((i) => (
            <div
              key={i}
              className="h-32 rounded-lg border border-border/50 bg-card/30 animate-pulse"
            />
          ))}
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
      <div className="space-y-2">
        <Label className="text-base">Theme</Label>
        <p className="text-sm text-muted-foreground">Choose your preferred appearance.</p>
      </div>
      <div className="grid grid-cols-3 gap-4">
        {THEMES.map((t) => {
          const isActive = theme === t.id;
          return (
            <button
              key={t.id}
              onClick={() => setTheme(t.id)}
              className={`relative group rounded-lg border-2 p-4 text-left transition-all duration-200 hover:scale-[1.02] ${
                isActive
                  ? "border-primary ring-2 ring-primary/20 shadow-lg"
                  : "border-border/50 hover:border-border"
              }`}
            >
              {/* Check badge */}
              {isActive && (
                <div className="absolute top-2 right-2 h-5 w-5 rounded-full bg-primary flex items-center justify-center">
                  <Check className="h-3 w-3 text-primary-foreground" />
                </div>
              )}

              {/* Color preview */}
              <div
                className="w-full h-16 rounded-md mb-3 overflow-hidden border border-black/10"
                style={{ backgroundColor: t.colors.bg }}
              >
                <div className="flex h-full">
                  <div className="w-1/4 h-full" style={{ backgroundColor: t.colors.card }} />
                  <div className="flex-1 p-2 flex flex-col justify-between">
                    <div
                      className="h-2 w-3/4 rounded-full"
                      style={{ backgroundColor: t.colors.primary }}
                    />
                    <div
                      className="h-1.5 w-1/2 rounded-full opacity-40"
                      style={{ backgroundColor: t.colors.accent }}
                    />
                    <div
                      className="h-1.5 w-2/3 rounded-full opacity-25"
                      style={{ backgroundColor: t.colors.accent }}
                    />
                  </div>
                </div>
              </div>

              {/* Label & desc */}
              <div className="flex items-center gap-2 mb-1">
                <t.icon className="h-4 w-4 text-muted-foreground" />
                <span className="text-sm font-semibold">{t.label}</span>
              </div>
              <p className="text-xs text-muted-foreground">{t.description}</p>
            </button>
          );
        })}
      </div>
    </div>
  );
}
