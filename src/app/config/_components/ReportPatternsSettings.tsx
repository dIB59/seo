"use client";

import { useState } from "react";
import useSWR from "swr";
import { Plus, Trash2, Pencil, Lock, X, Check } from "lucide-react";
import { toast } from "sonner";

import { Button } from "@/src/components/ui/button";
import { Input } from "@/src/components/ui/input";
import { Label } from "@/src/components/ui/label";
import { Switch } from "@/src/components/ui/switch";
import { Textarea } from "@/src/components/ui/textarea";
import { Badge } from "@/src/components/ui/badge";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/src/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/src/components/ui/select";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/src/components/ui/table";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/src/components/ui/tooltip";

import {
  listReportPatterns,
  createReportPattern,
  updateReportPattern,
  toggleReportPattern,
  deleteReportPattern,
  type ReportPattern,
  type ReportPatternParams,
} from "@/src/api/report";
import type { PatternCategory, PatternSeverity, BusinessImpact, FixEffort, Operator } from "@/src/bindings";

// ── Constants ──────────────────────────────────────────────────────────────

const CATEGORY_OPTIONS: { value: PatternCategory; label: string }[] = [
  { value: "technical", label: "Technical" },
  { value: "content", label: "Content" },
  { value: "performance", label: "Performance" },
  { value: "accessibility", label: "Accessibility" },
];

const SEVERITY_OPTIONS: { value: PatternSeverity; label: string }[] = [
  { value: "critical", label: "Critical" },
  { value: "warning", label: "Warning" },
  { value: "suggestion", label: "Suggestion" },
];

const OPERATOR_OPTIONS: { value: Operator; label: string; needsThreshold: boolean }[] = [
  { value: "missing", label: "is missing", needsThreshold: false },
  { value: "present", label: "is present", needsThreshold: false },
  { value: "eq", label: "equals", needsThreshold: true },
  { value: "lt", label: "less than", needsThreshold: true },
  { value: "gt", label: "greater than", needsThreshold: true },
  { value: "contains", label: "contains", needsThreshold: true },
  { value: "not_contains", label: "does not contain", needsThreshold: true },
];

const IMPACT_OPTIONS: { value: BusinessImpact; label: string }[] = [
  { value: "high", label: "High" },
  { value: "medium", label: "Medium" },
  { value: "low", label: "Low" },
];

const EFFORT_OPTIONS: { value: FixEffort; label: string }[] = [
  { value: "low", label: "Low" },
  { value: "medium", label: "Medium" },
  { value: "high", label: "High" },
];

const BUILT_IN_FIELDS = [
  "meta_description", "title", "canonical_url",
  "word_count", "load_time_ms", "status_code",
  "has_viewport", "has_structured_data", "h1_count",
];

const SEVERITY_BADGE: Record<PatternSeverity, "destructive" | "secondary" | "outline"> = {
  critical: "destructive",
  warning: "secondary",
  suggestion: "outline",
};

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

// ── Component ──────────────────────────────────────────────────────────────

export function ReportPatternsSettings() {
  const { data: patterns = [], mutate } = useSWR("report-patterns", listReportPatterns);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editing, setEditing] = useState<ReportPattern | null>(null);
  const [form, setForm] = useState<ReportPatternParams>(EMPTY_FORM);
  const [saving, setSaving] = useState(false);

  function openDialog(p?: ReportPattern) {
    setEditing(p ?? null);
    setForm(p ? {
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
    } : EMPTY_FORM);
    setDialogOpen(true);
  }

  async function handleSave() {
    if (!form.name.trim() || !form.field.trim() || !form.recommendation.trim()) {
      toast.error("Name, field, and recommendation are required");
      return;
    }
    setSaving(true);
    try {
      if (editing) {
        const updated = await updateReportPattern(editing.id, form);
        mutate((prev = []) => prev.map((p) => (p.id === editing.id ? updated : p)), false);
        toast.success("Pattern updated");
      } else {
        const created = await createReportPattern(form);
        mutate((prev = []) => [...prev, created], false);
        toast.success("Pattern created");
      }
      setDialogOpen(false);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Failed to save pattern");
    } finally {
      setSaving(false);
    }
  }

  async function handleDelete(id: string) {
    try {
      await deleteReportPattern(id);
      mutate((prev = []) => prev.filter((p) => p.id !== id), false);
      toast.success("Pattern deleted");
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Failed to delete pattern");
    }
  }

  async function handleToggle(p: ReportPattern) {
    try {
      await toggleReportPattern(p.id, !p.enabled);
      mutate((prev = []) => prev.map((x) => (x.id === p.id ? { ...x, enabled: !x.enabled } : x)), false);
    } catch {
      toast.error("Failed to toggle pattern");
    }
  }

  const opInfo = OPERATOR_OPTIONS.find((o) => o.value === form.operator);
  const needsThreshold = opInfo?.needsThreshold ?? false;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <p className="text-sm text-muted-foreground">
          Patterns are evaluated site-wide when generating a report. Built-in patterns can be
          disabled but not deleted. Custom patterns integrate with your extractors and custom checks.
        </p>
        <Button size="sm" onClick={() => openDialog()}>
          <Plus className="h-4 w-4 mr-1" />
          Add Pattern
        </Button>
      </div>

      {patterns.length === 0 ? (
        <p className="text-sm text-muted-foreground py-8 text-center">No patterns configured.</p>
      ) : (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Name</TableHead>
              <TableHead>Category</TableHead>
              <TableHead>Severity</TableHead>
              <TableHead>Condition</TableHead>
              <TableHead>Min %</TableHead>
              <TableHead>Enabled</TableHead>
              <TableHead className="w-20" />
            </TableRow>
          </TableHeader>
          <TableBody>
            {patterns.map((p) => (
              <TableRow key={p.id} className={!p.enabled ? "opacity-50" : undefined}>
                <TableCell className="font-medium">
                  <div className="flex items-center gap-1.5">
                    {p.isBuiltin && (
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Lock className="h-3 w-3 text-muted-foreground shrink-0" />
                        </TooltipTrigger>
                        <TooltipContent>Built-in pattern</TooltipContent>
                      </Tooltip>
                    )}
                    <span>{p.name}</span>
                  </div>
                </TableCell>
                <TableCell>
                  <Badge variant="outline" className="text-xs capitalize">
                    {p.category}
                  </Badge>
                </TableCell>
                <TableCell>
                  <Badge variant={SEVERITY_BADGE[p.severity]} className="text-xs capitalize">
                    {p.severity}
                  </Badge>
                </TableCell>
                <TableCell className="text-xs text-muted-foreground">
                  <code>{p.field}</code>{" "}
                  {OPERATOR_OPTIONS.find((o) => o.value === p.operator)?.label}
                  {p.threshold && <> <code>{p.threshold}</code></>}
                </TableCell>
                <TableCell className="text-xs text-muted-foreground">
                  {(p.minPrevalence * 100).toFixed(0)}%
                </TableCell>
                <TableCell>
                  <Switch checked={p.enabled} onCheckedChange={() => handleToggle(p)} />
                </TableCell>
                <TableCell>
                  <div className="flex gap-1">
                    {!p.isBuiltin && (
                      <>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-7 w-7"
                          onClick={() => openDialog(p)}
                        >
                          <Pencil className="h-3.5 w-3.5" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-7 w-7 text-destructive hover:text-destructive"
                          onClick={() => handleDelete(p.id)}
                        >
                          <Trash2 className="h-3.5 w-3.5" />
                        </Button>
                      </>
                    )}
                  </div>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      )}

      {/* Create / Edit dialog */}
      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <DialogContent className="sm:max-w-2xl">
          <DialogHeader>
            <DialogTitle>{editing ? "Edit Pattern" : "New Report Pattern"}</DialogTitle>
          </DialogHeader>

          <div className="grid grid-cols-2 gap-4 py-2">
            {/* Name */}
            <div className="space-y-1.5 col-span-2">
              <Label>Name</Label>
              <Input
                placeholder="Missing OG Images"
                value={form.name}
                onChange={(e) => setForm((f) => ({ ...f, name: e.target.value }))}
              />
            </div>

            {/* Description */}
            <div className="space-y-1.5 col-span-2">
              <Label>Description</Label>
              <Textarea
                placeholder="Brief explanation of what this pattern detects and why it matters."
                rows={2}
                value={form.description}
                onChange={(e) => setForm((f) => ({ ...f, description: e.target.value }))}
              />
            </div>

            {/* Category + Severity */}
            <div className="space-y-1.5">
              <Label>Category</Label>
              <Select
                value={form.category}
                onValueChange={(v) => setForm((f) => ({ ...f, category: v as PatternCategory }))}
              >
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  {CATEGORY_OPTIONS.map((o) => (
                    <SelectItem key={o.value} value={o.value}>{o.label}</SelectItem>
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
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  {SEVERITY_OPTIONS.map((o) => (
                    <SelectItem key={o.value} value={o.value}>{o.label}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            {/* Field */}
            <div className="space-y-1.5">
              <Label>Field</Label>
              <Input
                list="field-suggestions"
                placeholder="meta_description"
                value={form.field}
                onChange={(e) => setForm((f) => ({ ...f, field: e.target.value }))}
              />
              <datalist id="field-suggestions">
                {BUILT_IN_FIELDS.map((f) => <option key={f} value={f} />)}
              </datalist>
              <p className="text-xs text-muted-foreground">
                Built-in field, <code>extracted:&lt;key&gt;</code>, or <code>issue:&lt;type&gt;</code>
              </p>
            </div>

            {/* Operator */}
            <div className="space-y-1.5">
              <Label>Operator</Label>
              <Select
                value={form.operator}
                onValueChange={(v) =>
                  setForm((f) => ({ ...f, operator: v as Operator, threshold: null }))
                }
              >
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  {OPERATOR_OPTIONS.map((o) => (
                    <SelectItem key={o.value} value={o.value}>{o.label}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            {/* Threshold */}
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

            {/* Min Prevalence */}
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

            {/* Business Impact + Fix Effort */}
            <div className="space-y-1.5">
              <Label>Business Impact</Label>
              <Select
                value={form.businessImpact}
                onValueChange={(v) => setForm((f) => ({ ...f, businessImpact: v as BusinessImpact }))}
              >
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  {IMPACT_OPTIONS.map((o) => (
                    <SelectItem key={o.value} value={o.value}>{o.label}</SelectItem>
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
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  {EFFORT_OPTIONS.map((o) => (
                    <SelectItem key={o.value} value={o.value}>{o.label}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            {/* Recommendation */}
            <div className="space-y-1.5 col-span-2">
              <Label>Recommendation</Label>
              <Textarea
                placeholder="Describe how to fix this issue."
                rows={2}
                value={form.recommendation}
                onChange={(e) => setForm((f) => ({ ...f, recommendation: e.target.value }))}
              />
            </div>

            {/* Enabled */}
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
            <Button variant="ghost" onClick={() => setDialogOpen(false)}>
              <X className="h-4 w-4 mr-1" /> Cancel
            </Button>
            <Button onClick={handleSave} disabled={saving}>
              <Check className="h-4 w-4 mr-1" />
              {editing ? "Save Changes" : "Create"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
