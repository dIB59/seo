"use client";

import { useEffect, useState } from "react";
import useSWR from "swr";
import { listTags } from "@/src/api/extension";
import { Check, X } from "lucide-react";

import { Button } from "@/src/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/src/components/ui/dialog";
import { Input } from "@/src/components/ui/input";
import { Label } from "@/src/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/src/components/ui/select";
import { Switch } from "@/src/components/ui/switch";
import { Textarea } from "@/src/components/ui/textarea";

import type { ReportPattern, ReportPatternParams } from "@/src/api/report";
import type {
  BusinessImpact,
  FixEffort,
  Operator,
  PatternCategory,
  PatternSeverity,
} from "@/src/bindings";
import {
  CATEGORY_OPTIONS,
  EFFORT_OPTIONS,
  IMPACT_OPTIONS,
  OPERATOR_OPTIONS,
  SEVERITY_OPTIONS,
} from "./report-pattern-options";

const EMPTY_FORM: ReportPatternParams = {
  name: "",
  description: "",
  category: "technical",
  severity: "warning",
  field: "",
  operator: "missing",
  threshold: null,
  minPrevalence: 0.1,
  businessImpact: "medium",
  fixEffort: "medium",
  recommendation: "",
  enabled: true,
};

function paramsFrom(p: ReportPattern): ReportPatternParams {
  return {
    name: p.name,
    description: p.description,
    category: p.category,
    severity: p.severity,
    field: p.field,
    operator: p.operator,
    threshold: p.threshold,
    minPrevalence: p.minPrevalence,
    businessImpact: p.businessImpact,
    fixEffort: p.fixEffort,
    recommendation: p.recommendation,
    enabled: p.enabled,
  };
}

interface ReportPatternDialogProps {
  open: boolean;
  editing: ReportPattern | null;
  saving: boolean;
  onOpenChange: (open: boolean) => void;
  onSave: (params: ReportPatternParams) => void;
  onValidationError: (message: string) => void;
}

export function ReportPatternDialog({
  open,
  editing,
  saving,
  onOpenChange,
  onSave,
  onValidationError,
}: ReportPatternDialogProps) {
  const [form, setForm] = useState<ReportPatternParams>(EMPTY_FORM);
  const { data: tags = [] } = useSWR("tags-checkField", () => listTags("checkField"));

  useEffect(() => {
    if (!open) return;
    setForm(editing ? paramsFrom(editing) : EMPTY_FORM);
  }, [open, editing]);

  const needsThreshold =
    OPERATOR_OPTIONS.find((o) => o.value === form.operator)?.needsThreshold ?? false;

  function handleSave() {
    if (!form.name.trim() || !form.field.trim() || !form.recommendation.trim()) {
      onValidationError("Name, field, and recommendation are required");
      return;
    }
    onSave(form);
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>{editing ? "Edit Pattern" : "New Report Pattern"}</DialogTitle>
        </DialogHeader>

        <div className="grid grid-cols-2 gap-4 py-2">
          <div className="space-y-1.5 col-span-2">
            <Label>Name</Label>
            <Input
              placeholder="Missing OG Images"
              value={form.name}
              onChange={(e) => setForm((f) => ({ ...f, name: e.target.value }))}
            />
          </div>

          <div className="space-y-1.5 col-span-2">
            <Label>Description</Label>
            <Textarea
              placeholder="Brief explanation of what this pattern detects and why it matters."
              rows={2}
              value={form.description}
              onChange={(e) => setForm((f) => ({ ...f, description: e.target.value }))}
            />
          </div>

          <div className="space-y-1.5">
            <Label>Category</Label>
            <Select
              value={form.category}
              onValueChange={(v) => setForm((f) => ({ ...f, category: v as PatternCategory }))}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {CATEGORY_OPTIONS.map((o) => (
                  <SelectItem key={o.value} value={o.value}>
                    {o.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-1.5">
            <Label>Severity</Label>
            <Select
              value={form.severity}
              onValueChange={(v) => setForm((f) => ({ ...f, severity: v as PatternSeverity }))}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {SEVERITY_OPTIONS.map((o) => (
                  <SelectItem key={o.value} value={o.value}>
                    {o.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-1.5">
            <Label>Field</Label>
            <Select
              value={form.field}
              onValueChange={(v) => setForm((f) => ({ ...f, field: v }))}
            >
              <SelectTrigger>
                <SelectValue placeholder="Select a field…" />
              </SelectTrigger>
              <SelectContent>
                {tags.map((t) => (
                  <SelectItem key={t.name} value={t.name}>
                    <span className="flex items-center gap-2">
                      <code className="text-xs">{t.name}</code>
                      <span className="text-xs text-muted-foreground">{t.label}</span>
                    </span>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-1.5">
            <Label>Operator</Label>
            <Select
              value={form.operator}
              onValueChange={(v) =>
                setForm((f) => ({ ...f, operator: v as Operator, threshold: null }))
              }
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {OPERATOR_OPTIONS.map((o) => (
                  <SelectItem key={o.value} value={o.value}>
                    {o.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {needsThreshold && (
            <div className="space-y-1.5">
              <Label>Threshold</Label>
              <Input
                placeholder={form.operator === "lt" || form.operator === "gt" ? "300" : "value"}
                value={form.threshold ?? ""}
                onChange={(e) =>
                  setForm((f) => ({ ...f, threshold: e.target.value || null }))
                }
              />
            </div>
          )}

          <div className="space-y-1.5">
            <Label>Min Prevalence (%)</Label>
            <Input
              type="number"
              min={0}
              max={100}
              step={1}
              placeholder="10"
              value={Math.round(form.minPrevalence * 100)}
              onChange={(e) =>
                setForm((f) => ({ ...f, minPrevalence: Number(e.target.value) / 100 }))
              }
            />
            <p className="text-xs text-muted-foreground">
              Pattern fires when at least this % of pages match.
            </p>
          </div>

          <div className="space-y-1.5">
            <Label>Business Impact</Label>
            <Select
              value={form.businessImpact}
              onValueChange={(v) =>
                setForm((f) => ({ ...f, businessImpact: v as BusinessImpact }))
              }
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {IMPACT_OPTIONS.map((o) => (
                  <SelectItem key={o.value} value={o.value}>
                    {o.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-1.5">
            <Label>Fix Effort</Label>
            <Select
              value={form.fixEffort}
              onValueChange={(v) => setForm((f) => ({ ...f, fixEffort: v as FixEffort }))}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {EFFORT_OPTIONS.map((o) => (
                  <SelectItem key={o.value} value={o.value}>
                    {o.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-1.5 col-span-2">
            <Label>Recommendation</Label>
            <Textarea
              placeholder="Describe how to fix this issue."
              rows={2}
              value={form.recommendation}
              onChange={(e) => setForm((f) => ({ ...f, recommendation: e.target.value }))}
            />
          </div>

          <div className="col-span-2">
            <label className="flex items-center gap-2 text-sm cursor-pointer">
              <Switch
                checked={form.enabled}
                onCheckedChange={(v) => setForm((f) => ({ ...f, enabled: v }))}
              />
              Enabled
            </label>
          </div>
        </div>

        <DialogFooter>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            <X className="h-4 w-4 mr-1" /> Cancel
          </Button>
          <Button onClick={handleSave} disabled={saving}>
            <Check className="h-4 w-4 mr-1" />
            {editing ? "Save Changes" : "Create"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
