// Design tokens — professional, muted, monochromatic-first.
// Accent colours are used only for score/severity indicators, never as fills.

export const C = {
  // Page
  white: "#FFFFFF",
  pageBg: "#FFFFFF",

  // Dark page frame (top bar on every page)
  ink: "#111118",
  inkSurface: "#1C1C26",
  inkText: "#FFFFFF",
  inkMuted: "#8B8FA8",

  // Body typography
  text: "#111118",
  secondary: "#4B5563",   // gray-600
  muted: "#6B7280",       // gray-500
  faint: "#9CA3AF",       // gray-400
  hairline: "#9CA3AF",    // very light divider

  // Surfaces
  surface: "#F9FAFB",     // gray-50
  border: "#E5E7EB",      // gray-200

  // Severity — all intentionally muted/dark, never bright
  critical: "#B91C1C",    // red-700
  warning: "#B45309",     // amber-700
  suggestion: "#6D28D9",  // violet-700
  success: "#15803D",     // green-700

  // Pillar bars — desaturated, readable
  pillarTechnical:    "#4B7DB8",
  pillarContent:      "#7C5ABF",
  pillarPerformance:  "#B07A1E",
  pillarAccessibility:"#2D8A52",

  // Score colour thresholds
  scoreGood:    "#15803D",
  scoreWarn:    "#B45309",
  scoreBad:     "#B91C1C",
} as const;

export const F = {
  regular: "Helvetica",
  bold:    "Helvetica-Bold",
  italic:  "Helvetica-Oblique",
  mono:    "Courier",
} as const;

// Spacing
export const SP = {
  page:    40,   // horizontal page margin
  bar:     32,   // top bar height
  gap: {
    xs:  3,
    sm:  6,
    md:  10,
    lg:  16,
    xl:  24,
    xxl: 36,
  },
} as const;

// ── Helpers ───────────────────────────────────────────────────────────────────

export function scoreColor(n: number): string {
  if (n >= 70) return C.scoreGood;
  if (n >= 40) return C.scoreWarn;
  return C.scoreBad;
}

export function scoreGrade(n: number): string {
  if (n >= 90) return "Excellent";
  if (n >= 70) return "Good";
  if (n >= 50) return "Needs Attention";
  if (n >= 30) return "Poor";
  return "Critical";
}

export function severityColor(s: string): string {
  switch (s) {
    case "critical":   return C.critical;
    case "warning":    return C.warning;
    case "suggestion": return C.suggestion;
    default:           return C.faint;
  }
}

export function pillarColor(cat: string): string {
  switch (cat.toLowerCase()) {
    case "technical":    return C.pillarTechnical;
    case "content":      return C.pillarContent;
    case "performance":  return C.pillarPerformance;
    case "accessibility":return C.pillarAccessibility;
    default:             return C.muted;
  }
}

// Strip basic markdown syntax for plain-text rendering
export function stripMd(s: string): string {
  return s
    .replace(/\*\*(.+?)\*\*/g, "$1")
    .replace(/\*(.+?)\*/g, "$1")
    .replace(/_(.+?)_/g, "$1")
    .replace(/`(.+?)`/g, "$1");
}
