import type {
  BusinessImpact,
  FixEffort,
  Operator,
  PatternCategory,
  PatternSeverity,
} from "@/src/bindings";

export const CATEGORY_OPTIONS: { value: PatternCategory; label: string }[] = [
  { value: "technical", label: "Technical" },
  { value: "content", label: "Content" },
  { value: "performance", label: "Performance" },
  { value: "accessibility", label: "Accessibility" },
];

export const SEVERITY_OPTIONS: { value: PatternSeverity; label: string }[] = [
  { value: "critical", label: "Critical" },
  { value: "warning", label: "Warning" },
  { value: "suggestion", label: "Suggestion" },
];

export const OPERATOR_OPTIONS: { value: Operator; label: string; needsThreshold: boolean }[] = [
  { value: "missing", label: "is missing", needsThreshold: false },
  { value: "present", label: "is present", needsThreshold: false },
  { value: "eq", label: "equals", needsThreshold: true },
  { value: "lt", label: "less than", needsThreshold: true },
  { value: "gt", label: "greater than", needsThreshold: true },
  { value: "contains", label: "contains", needsThreshold: true },
  { value: "not_contains", label: "does not contain", needsThreshold: true },
];

export const IMPACT_OPTIONS: { value: BusinessImpact; label: string }[] = [
  { value: "high", label: "High" },
  { value: "medium", label: "Medium" },
  { value: "low", label: "Low" },
];

export const EFFORT_OPTIONS: { value: FixEffort; label: string }[] = [
  { value: "low", label: "Low" },
  { value: "medium", label: "Medium" },
  { value: "high", label: "High" },
];