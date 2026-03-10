import type { LucideIcon } from "lucide-react";
import { Settings2, Eye, Gauge, Hash } from "lucide-react";
import type { ExtensionCategory, RuleSeverity, RuleType } from "@/src/lib/types/extension";

export interface RuleTemplate {
  id: string;
  name: string;
  description: string;
  category: ExtensionCategory;
  ruleType: RuleType;
  targetField: string;
  thresholdMin?: string;
  thresholdMax?: string;
  regexPattern?: string;
  recommendation: string;
  severity: RuleSeverity;
  icon: LucideIcon;
}

export interface TargetField {
  value: string;
  label: string;
  description: string;
}

export interface CategoryConfig {
  label: string;
  accent: string;
  lightBg: string;
}

export interface RuleTypeConfig {
  label: string;
  description: string;
  icon: LucideIcon;
}

export const CATEGORY_CONFIG: Record<string, CategoryConfig> = {
  seo: { label: "SEO", accent: "text-chart-1", lightBg: "bg-chart-1/10" },
  accessibility: { label: "Accessibility", accent: "text-chart-2", lightBg: "bg-chart-2/10" },
  performance: { label: "Performance", accent: "text-chart-3", lightBg: "bg-chart-3/10" },
  security: { label: "Security", accent: "text-destructive", lightBg: "bg-destructive/10" },
  content: { label: "Content", accent: "text-chart-4", lightBg: "bg-chart-4/10" },
  technical: { label: "Technical", accent: "text-muted-foreground", lightBg: "bg-muted" },
  ux: { label: "UX", accent: "text-chart-5", lightBg: "bg-chart-5/10" },
  mobile: { label: "Mobile", accent: "text-chart-1", lightBg: "bg-chart-1/10" },
};

export const RULE_TYPE_CONFIG: Record<string, RuleTypeConfig> = {
  presence: {
    label: "Presence Check",
    description: "Verifies that a field exists on the page",
    icon: Eye,
  },
  threshold: {
    label: "Threshold Check",
    description: "Checks if a value is within acceptable bounds",
    icon: Gauge,
  },
  regex: { label: "Pattern Match", description: "Validates against a regex pattern", icon: Hash },
  custom: { label: "Custom Rule", description: "Create custom validation logic", icon: Settings2 },
};

export const SEVERITIES: RuleSeverity[] = ["critical", "warning", "info"];

export const RULE_TYPES: RuleType[] = ["presence", "threshold", "regex", "custom"];

export const CUSTOM_FIELD_VALUE = "__custom__";
export const CUSTOM_CATEGORY_VALUE = "__custom_category__";
