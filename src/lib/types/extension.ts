export type RuleType = "presence" | "threshold" | "regex" | "custom";

export type RuleSeverity = "critical" | "warning" | "info";
export const EXTENSION_CATEGORIES = [
  "seo",
  "accessibility",
  "performance",
  "security",
  "content",
  "technical",
  "ux",
  "mobile",
] as const;

export type ExtensionCategory = (typeof EXTENSION_CATEGORIES)[number];
