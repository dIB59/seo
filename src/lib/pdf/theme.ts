// Design tokens for the SEO Insikt PDF report.
// Mirrors the app's visual language: near-black surfaces, indigo accent,
// generous whitespace, and semantic colors used sparingly.

export const C = {
  // Page
  pageBg: "#FFFFFF",

  // Dark cover / header band
  ink: "#0C0D11",        // near-black background
  inkSurface: "#15171E", // slightly lighter surface on dark
  inkBorder: "#2A2D3A",  // subtle dark border

  // Body surfaces
  surface: "#F8F9FC",    // off-white card background
  border: "#E8EAF0",     // hairline divider
  borderMid: "#D1D5E0",  // slightly more visible border

  // Typography
  text: "#0C0D11",       // primary body text
  muted: "#6B7280",      // secondary/label text
  faint: "#9CA3AF",      // tertiary/hint text
  onDark: "#FFFFFF",     // text on dark background
  onDarkMuted: "#9AA3B8",// muted text on dark background

  // Brand
  accent: "#6366F1",     // indigo-500 — primary brand
  accentLight: "#EEF2FF",// indigo-50

  // Semantic
  critical: "#EF4444",
  criticalBg: "#FEF2F2",
  warning: "#F59E0B",
  warningBg: "#FFFBEB",
  success: "#22C55E",
  successBg: "#F0FDF4",
  info: "#3B82F6",
  infoBg: "#EFF6FF",
  suggestion: "#8B5CF6",
  suggestionBg: "#F5F3FF",
} as const;

export const F = {
  regular: "Helvetica",
  bold: "Helvetica-Bold",
  italic: "Helvetica-Oblique",
  mono: "Courier",
} as const;

export const S = {
  pageH: 841.89,  // A4 height in pt
  pageW: 595.28,  // A4 width in pt
  margin: 36,
  marginLg: 48,
  gap: {
    xs: 3,
    sm: 6,
    md: 10,
    lg: 16,
    xl: 24,
    xxl: 36,
  },
} as const;

// Severity helpers
export function severityColor(sev: string) {
  switch (sev) {
    case "critical":   return C.critical;
    case "warning":    return C.warning;
    case "suggestion": return C.suggestion;
    default:           return C.faint;
  }
}

export function severityBg(sev: string) {
  switch (sev) {
    case "critical":   return C.criticalBg;
    case "warning":    return C.warningBg;
    case "suggestion": return C.suggestionBg;
    default:           return C.surface;
  }
}

export function pillarColor(cat: string) {
  switch (cat) {
    case "technical":    return C.info;
    case "content":      return C.accent;
    case "performance":  return C.warning;
    case "accessibility":return C.success;
    default:             return C.faint;
  }
}

export function scoreColor(n: number) {
  if (n >= 80) return C.success;
  if (n >= 50) return C.warning;
  return C.critical;
}

export function scoreLabel(n: number) {
  if (n >= 90) return "Excellent";
  if (n >= 80) return "Good";
  if (n >= 60) return "Needs Work";
  if (n >= 40) return "Poor";
  return "Critical";
}
