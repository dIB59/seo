"use client";

import type { LucideIcon } from "lucide-react";
import { Check, ArrowRight } from "lucide-react";
import { Badge } from "@/src/components/ui/badge";
import { CATEGORY_CONFIG, type RuleTemplate, type CategoryConfig } from "./rule-config";

// ============================================================================
// Step Component
// ============================================================================

interface StepProps {
  number: number;
  title: string;
  isActive: boolean;
  isCompleted: boolean;
}

export function Step({ number, title, isActive, isCompleted }: StepProps) {
  return (
    <div className="flex items-center gap-3">
      <div
        className={`w-8 h-8 rounded-lg flex items-center justify-center text-sm font-bold transition-all duration-300 ${
          isCompleted
            ? "bg-success text-success-foreground shadow-lg shadow-success/25"
            : isActive
              ? "bg-foreground text-background shadow-lg"
              : "bg-muted text-muted-foreground"
        }`}
      >
        {isCompleted ? <Check className="h-4 w-4" /> : number}
      </div>
      <span
        className={`text-sm font-medium transition-colors ${isActive ? "text-foreground" : "text-muted-foreground"}`}
      >
        {title}
      </span>
    </div>
  );
}

// ============================================================================
// Template Card Component
// ============================================================================

interface TemplateCardProps {
  template: RuleTemplate;
  onSelect: (template: RuleTemplate) => void;
}

export function TemplateCard({ template, onSelect }: TemplateCardProps) {
  const catConfig: CategoryConfig = CATEGORY_CONFIG[template.category] || CATEGORY_CONFIG.technical;
  const IconComponent: LucideIcon = template.icon;

  return (
    <button
      type="button"
      onClick={() => onSelect(template)}
      className="w-full p-4 rounded-2xl border border-border/60 bg-gradient-to-br from-background to-muted/30 hover:from-background hover:to-background hover:border-primary/20 hover:shadow-lg hover:shadow-primary/5 transition-all duration-300 text-left group relative overflow-hidden"
    >
      <div className="absolute inset-0 bg-gradient-to-r from-primary/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
      <div className="flex items-start gap-4 relative">
        <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-muted to-muted/50 flex items-center justify-center border border-border/60 shadow-sm group-hover:scale-110 transition-transform duration-300">
          <IconComponent className="h-5 w-5 text-primary" />
        </div>
        <div className="flex-1 min-w-0">
          <h4 className="font-semibold text-foreground text-[15px] group-hover:text-primary transition-colors">
            {template.name}
          </h4>
          <p className="text-sm text-muted-foreground mt-1.5 line-clamp-2 leading-relaxed">
            {template.description}
          </p>
          <div className="flex items-center gap-2 mt-3">
            <span
              className={`text-xs font-medium px-2 py-0.5 rounded-full ${catConfig.lightBg} ${catConfig.accent}`}
            >
              {catConfig.label}
            </span>
            <span className="text-border">•</span>
            <span className="text-xs text-muted-foreground capitalize">{template.ruleType}</span>
            <span className="text-border">•</span>
            <Badge
              variant={
                template.severity === "critical"
                  ? "destructive"
                  : template.severity === "warning"
                    ? "default"
                    : "secondary"
              }
              className="text-[10px] px-2 py-0.5 h-5 capitalize"
            >
              {template.severity}
            </Badge>
          </div>
        </div>
        <ArrowRight className="h-4 w-4 text-muted-foreground opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-300" />
      </div>
    </button>
  );
}

// ============================================================================
// Preview Panel
// ============================================================================

interface PreviewPanelProps {
  name: string;
  category: string;
  severity: string;
  ruleType: string;
  recommendation: string;
  thresholdMin: string;
  thresholdMax: string;
  regexPattern: string;
}

export function PreviewPanel({
  name,
  category,
  severity,
  ruleType,
  recommendation,
  thresholdMin,
  thresholdMax,
  regexPattern,
}: PreviewPanelProps) {
  const catConfig: CategoryConfig = CATEGORY_CONFIG[category] || CATEGORY_CONFIG.technical;
  const typeLabel = RULE_TYPE_CONFIG[ruleType]?.label || ruleType;

  return (
    <div className="rounded-xl border border-border/60 bg-muted/30 overflow-hidden">
      <div className="px-4 py-2.5 bg-muted/50 border-b border-border/40 flex items-center gap-2">
        <span className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
          Preview
        </span>
      </div>
      <div className="p-4 space-y-3">
        <div className="flex items-center gap-2.5">
          <Badge
            variant={
              severity === "critical"
                ? "destructive"
                : severity === "warning"
                  ? "default"
                  : "secondary"
            }
            className="capitalize text-xs"
          >
            {severity}
          </Badge>
          <span className="font-semibold text-foreground">{name || "Your Rule Name"}</span>
        </div>

        <p className="text-sm text-muted-foreground leading-relaxed">
          {recommendation || "Add a recommendation to guide users on how to fix this issue..."}
        </p>

        <div className="flex items-center gap-3 pt-1.5 border-t border-border/40">
          <div className="flex items-center gap-1.5">
            <div className={`w-1.5 h-1.5 rounded-full ${catConfig.lightBg}`} />
            <span className="text-[11px] font-medium text-muted-foreground">{catConfig.label}</span>
          </div>
          <span className="text-border">•</span>
          <span className="text-[11px] text-muted-foreground">{typeLabel}</span>
        </div>

        {(thresholdMin || thresholdMax || regexPattern) && (
          <div className="flex flex-wrap gap-1.5 pt-2 border-t border-border/40">
            {thresholdMin && (
              <Badge variant="outline" className="text-[10px]">
                {thresholdMin} min
              </Badge>
            )}
            {thresholdMax && (
              <Badge variant="outline" className="text-[10px]">
                {thresholdMax} max
              </Badge>
            )}
            {regexPattern && (
              <Badge variant="outline" className="text-[10px] font-mono">
                {regexPattern.slice(0, 12)}
              </Badge>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

// ============================================================================
// Changes Tracker
// ============================================================================

interface ChangesTrackerProps {
  original: {
    name?: string;
    severity?: string;
    recommendation?: string;
    thresholdMin?: number;
    thresholdMax?: number;
    regexPattern?: string;
  };
  current: {
    name: string;
    severity: string;
    recommendation: string;
    thresholdMin: string;
    thresholdMax: string;
    regexPattern: string;
  };
}

export function ChangesTracker({ original, current }: ChangesTrackerProps) {
  const hasChanges =
    current.name !== (original.name || "") ||
    current.severity !== original.severity ||
    current.recommendation !== (original.recommendation || "") ||
    current.thresholdMin !== (original.thresholdMin?.toString() || "") ||
    current.thresholdMax !== (original.thresholdMax?.toString() || "");

  if (!hasChanges) return null;

  return (
    <div className="rounded-xl border border-warning/30 bg-warning/5 overflow-hidden">
      <div className="px-4 py-2.5 bg-warning/10 border-b border-warning/20 flex items-center gap-2">
        <span className="text-xs font-medium uppercase tracking-wide text-warning dark:text-warning">
          Modified
        </span>
      </div>
      <div className="p-3 space-y-2">
        {current.name !== (original.name || "") && (
          <div className="flex items-center gap-2 text-xs">
            <span className="line-through opacity-50 w-16 shrink-0">Name</span>
            <ArrowRight className="h-2.5 w-2.5 text-muted-foreground shrink-0" />
            <span className="font-medium">{current.name}</span>
          </div>
        )}
        {current.severity !== original.severity && (
          <div className="flex items-center gap-2 text-xs">
            <span className="line-through opacity-50 w-16 shrink-0">Severity</span>
            <ArrowRight className="h-2.5 w-2.5 text-muted-foreground shrink-0" />
            <Badge variant="outline" className="text-[10px] capitalize">
              {current.severity}
            </Badge>
          </div>
        )}
      </div>
    </div>
  );
}
