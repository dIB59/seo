// Design tokens for the SEO Insikt PDF report.
// All values are compatible with @react-pdf/renderer style props.

export const COLORS = {
  // Brand
  primary: "#6366f1",      // indigo-500
  primaryDark: "#4f46e5",  // indigo-600
  primaryLight: "#e0e7ff", // indigo-100

  // Neutrals
  white: "#ffffff",
  gray50: "#f9fafb",
  gray100: "#f3f4f6",
  gray200: "#e5e7eb",
  gray400: "#9ca3af",
  gray500: "#6b7280",
  gray600: "#4b5563",
  gray700: "#374151",
  gray900: "#111827",

  // Semantic
  success: "#22c55e",   // green-500
  warning: "#f59e0b",   // amber-500
  danger: "#ef4444",    // red-500
  info: "#3b82f6",      // blue-500

  // Severity
  critical: "#ef4444",
  warning2: "#f59e0b",
  suggestion: "#6366f1",
} as const;

export const FONTS = {
  regular: "Helvetica",
  bold: "Helvetica-Bold",
  italic: "Helvetica-Oblique",
} as const;

export const SPACING = {
  xs: 4,
  sm: 8,
  md: 12,
  lg: 16,
  xl: 24,
  xxl: 32,
  page: 40,
} as const;

export function severityColor(severity: string): string {
  switch (severity) {
    case "critical": return COLORS.critical;
    case "warning": return COLORS.warning2;
    default: return COLORS.suggestion;
  }
}

export function pillarColor(pillar: string): string {
  switch (pillar) {
    case "technical": return COLORS.info;
    case "content": return COLORS.primary;
    case "performance": return COLORS.warning2;
    case "accessibility": return COLORS.success;
    default: return COLORS.gray400;
  }
}

export function scoreColor(score: number): string {
  if (score >= 80) return COLORS.success;
  if (score >= 50) return COLORS.warning2;
  return COLORS.danger;
}
