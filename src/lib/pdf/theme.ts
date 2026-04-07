// Design tokens for the PDF report.
//
// Aligns with the SEO Insikt app: Geist Sans / Geist Mono, blue primary
// accent (sRGB approximation of the app's `oklch(0.55 0.15 250)`), soft
// rounded surfaces. Colour is reserved for score / severity / accent —
// never used as a heavy fill.

export const C = {
  // Page
  white: "#FFFFFF",
  pageBg: "#FFFFFF",

  // Brand accent — derived from app primary `oklch(0.55 0.15 250)`
  primary:     "#2F6FE6",
  primarySoft: "#EAF1FD",
  primaryDim:  "#5B86E0",   // section labels / kickers

  // Dark page frame (top bar on interior pages)
  ink: "#111118",
  inkSurface: "#1C1C26",
  inkText: "#FFFFFF",
  inkMuted: "#8B8FA8",

  // Body typography
  text: "#111118",
  secondary: "#4B5563",   // gray-600
  muted: "#6B7280",       // gray-500
  faint: "#9CA3AF",       // gray-400
  hairline: "#9CA3AF",

  // Surfaces
  surface: "#F7F8FB",     // softer than gray-50
  surfaceAlt: "#F1F3F8",  // for nested cards
  border: "#E6E8EF",      // softer gray-200

  // Severity — muted, never bright
  critical: "#B91C1C",
  warning:  "#B45309",
  suggestion: "#6D28D9",
  success:  "#15803D",

  // Pillar bars — desaturated, readable
  pillarTechnical:    "#4B7DB8",
  pillarContent:      "#7C5ABF",
  pillarPerformance:  "#B07A1E",
  pillarAccessibility:"#2D8A52",

  // Score colour thresholds
  scoreGood: "#15803D",
  scoreWarn: "#B45309",
  scoreBad:  "#B91C1C",
} as const;

// Font family names — registered in `generate.ts` via Font.register.
// Inter for sans (Geist's GSUB ligatures collapse in react-pdf),
// JetBrains Mono for monospace data.
export const F = {
  regular: "Inter",
  medium:  "Inter-Medium",
  bold:    "Inter-Bold",
  italic:  "Inter",
  mono:    "JetBrainsMono",
  monoMed: "JetBrainsMono-Medium",
} as const;

// Spacing
export const SP = {
  page: 40,
  bar:  32,
  gap: {
    xs:  3,
    sm:  6,
    md:  10,
    lg:  16,
    xl:  24,
    xxl: 36,
  },
  radius: 6,
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
