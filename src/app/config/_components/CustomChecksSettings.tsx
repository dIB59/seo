"use client";

import { useState } from "react";
import useSWR from "swr";
import { Plus, Trash2, Pencil, X, Check } from "lucide-react";
import { toast } from "sonner";

import { Button } from "@/src/components/ui/button";
import { Input } from "@/src/components/ui/input";
import { Label } from "@/src/components/ui/label";
import { Switch } from "@/src/components/ui/switch";
import { Textarea } from "@/src/components/ui/textarea";
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
import { Badge } from "@/src/components/ui/badge";

import {
  listCustomChecks,
  createCustomCheck,
  updateCustomCheck,
  deleteCustomCheck,
  type CustomCheck,
  type CustomCheckParams,
} from "@/src/api/extension";

type Severity = "info" | "warning" | "critical";
type Operator = "missing" | "lt" | "gt" | "contains" | "not_contains";

const SEVERITY_OPTIONS: { value: Severity; label: string }[] = [
  { value: "info", label: "Info" },
  { value: "warning", label: "Warning" },
  { value: "critical", label: "Critical" },
];

const OPERATOR_OPTIONS: { value: Operator; label: string }[] = [
  { value: "missing", label: "is missing" },
  { value: "lt", label: "less than" },
  { value: "gt", label: "greater than" },
  { value: "contains", label: "contains" },
  { value: "not_contains", label: "does not contain" },
];

const SEVERITY_VARIANT: Record<Severity, "destructive" | "secondary" | "outline"> = {
  critical: "destructive",
  warning: "secondary",
  info: "outline",
};

const EMPTY_PARAMS: CustomCheckParams = {
  name: "",
  severity: "warning",
  field: "",
  operator: "missing",
  threshold: null,
  message_template: "",
  enabled: true,
};

export function CustomChecksSettings() {
  const { data: checks = [], mutate } = useSWR("custom-checks", listCustomChecks);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editing, setEditing] = useState<CustomCheck | null>(null);
  const [form, setForm] = useState<CustomCheckParams>(EMPTY_PARAMS);
  const [saving, setSaving] = useState(false);

  function openCreate() {
    setEditing(null);
    setForm(EMPTY_PARAMS);
    setDialogOpen(true);
  }

  function openEdit(check: CustomCheck) {
    setEditing(check);
    setForm({
      name: check.name,
      severity: check.severity,
      field: check.field,
      operator: check.operator,
      threshold: check.threshold,
      message_template: check.message_template,
      enabled: check.enabled,
    });
    setDialogOpen(true);
  }

  async function handleSave() {
    if (!form.name.trim() || !form.field.trim() || !form.message_template.trim()) {
      toast.error("Name, field, and message template are required");
      return;
    }
    setSaving(true);
    try {
      if (editing) {
        const updated = await updateCustomCheck(editing.id, form);
        mutate((prev = []) => prev.map((c) => (c.id === editing.id ? updated : c)), false);
        toast.success("Check updated");
      } else {
        const created = await createCustomCheck(form);
        mutate((prev = []) => [...prev, created], false);
        toast.success("Check created");
      }
      setDialogOpen(false);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Failed to save check");
    } finally {
      setSaving(false);
    }
  }

  async function handleDelete(id: string) {
    try {
      await deleteCustomCheck(id);
      mutate((prev = []) => prev.filter((c) => c.id !== id), false);
      toast.success("Check deleted");
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Failed to delete check");
    }
  }

  async function handleToggleEnabled(check: CustomCheck) {
    try {
      const updated = await updateCustomCheck(check.id, {
        ...check,
        threshold: check.threshold ?? null,
        enabled: !check.enabled,
      });
      mutate((prev = []) => prev.map((c) => (c.id === check.id ? updated : c)), false);
    } catch {
      toast.error("Failed to toggle check");
    }
  }

  const needsThreshold = form.operator !== "missing";

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <p className="text-sm text-muted-foreground">
          Custom checks run on every crawled page and appear alongside built-in SEO issues.
        </p>
        <Button size="sm" onClick={openCreate}>
          <Plus className="h-4 w-4 mr-1" />
          Add Check
        </Button>
      </div>

      {checks.length === 0 ? (
        <p className="text-sm text-muted-foreground py-8 text-center">No custom checks configured.</p>
      ) : (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Name</TableHead>
              <TableHead>Severity</TableHead>
              <TableHead>Condition</TableHead>
              <TableHead>Enabled</TableHead>
              <TableHead className="w-20" />
            </TableRow>
          </TableHeader>
          <TableBody>
            {checks.map((c) => (
              <TableRow key={c.id}>
                <TableCell className="font-medium">{c.name}</TableCell>
                <TableCell>
                  <Badge variant={SEVERITY_VARIANT[c.severity as Severity]} className="text-xs capitalize">
                    {c.severity}
                  </Badge>
                </TableCell>
                <TableCell className="text-sm text-muted-foreground">
                  <code className="text-xs">{c.field}</code>{" "}
                  {OPERATOR_OPTIONS.find((o) => o.value === c.operator)?.label}
                  {c.threshold && <> {c.threshold}</>}
                </TableCell>
                <TableCell>
                  <Switch
                    checked={c.enabled}
                    onCheckedChange={() => handleToggleEnabled(c)}
                  />
                </TableCell>
                <TableCell>
                  <div className="flex gap-1">
                    <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => openEdit(c)}>
                      <Pencil className="h-3.5 w-3.5" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-7 w-7 text-destructive hover:text-destructive"
                      onClick={() => handleDelete(c.id)}
                    >
                      <Trash2 className="h-3.5 w-3.5" />
                    </Button>
                  </div>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      )}

      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <DialogContent className="sm:max-w-lg">
          <DialogHeader>
            <DialogTitle>{editing ? "Edit Check" : "New Custom Check"}</DialogTitle>
          </DialogHeader>
          <div className="space-y-4 py-2">
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-1.5">
                <Label htmlFor="chk-name">Name</Label>
                <Input
                  id="chk-name"
                  placeholder="Missing Schema"
                  value={form.name}
                  onChange={(e) => setForm((f) => ({ ...f, name: e.target.value }))}
                />
              </div>
              <div className="space-y-1.5">
                <Label>Severity</Label>
                <Select
                  value={form.severity as Severity}
                  onValueChange={(v) => setForm((f) => ({ ...f, severity: v as Severity }))}
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
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-1.5">
                <Label htmlFor="chk-field">Field</Label>
                <Input
                  id="chk-field"
                  placeholder="og_image"
                  value={form.field}
                  onChange={(e) => setForm((f) => ({ ...f, field: e.target.value }))}
                />
                <p className="text-xs text-muted-foreground">
                  Key from extracted_data or built-in page field
                </p>
              </div>
              <div className="space-y-1.5">
                <Label>Operator</Label>
                <Select
                  value={form.operator as Operator}
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
            </div>

            {needsThreshold && (
              <div className="space-y-1.5">
                <Label htmlFor="chk-threshold">Threshold</Label>
                <Input
                  id="chk-threshold"
                  placeholder={form.operator === "lt" || form.operator === "gt" ? "300" : "keyword"}
                  value={form.threshold ?? ""}
                  onChange={(e) =>
                    setForm((f) => ({ ...f, threshold: e.target.value || null }))
                  }
                />
              </div>
            )}

            <div className="space-y-1.5">
              <Label htmlFor="chk-msg">Message Template</Label>
              <Textarea
                id="chk-msg"
                placeholder="Page is missing an OG image. Use {value} to show the actual value."
                rows={2}
                value={form.message_template}
                onChange={(e) => setForm((f) => ({ ...f, message_template: e.target.value }))}
              />
            </div>

            <label className="flex items-center gap-2 text-sm cursor-pointer">
              <Switch
                checked={form.enabled}
                onCheckedChange={(v) => setForm((f) => ({ ...f, enabled: v }))}
              />
              Enabled
            </label>
          </div>
          <DialogFooter>
            <Button variant="ghost" onClick={() => setDialogOpen(false)}>
              <X className="h-4 w-4 mr-1" />
              Cancel
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
