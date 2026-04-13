"use client";

import { useState } from "react";
import { useCheckFieldTags } from "@/src/hooks/use-check-field-tags";
import { useFormSync } from "@/src/hooks/use-form-sync";
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

import type { CustomCheck, CustomCheckParams } from "@/src/api/extension";
import {
  OPERATOR_OPTIONS,
  SEVERITY_OPTIONS,
  type CheckOperator,
  type CheckSeverity,
} from "./custom-check-options";

const EMPTY_PARAMS: CustomCheckParams = {
  name: "",
  severity: "warning",
  field: "",
  operator: "missing",
  threshold: null,
  message_template: "",
  enabled: true,
};

function paramsFrom(check: CustomCheck): CustomCheckParams {
  return {
    name: check.name,
    severity: check.severity,
    field: check.field,
    operator: check.operator,
    threshold: check.threshold,
    message_template: check.message_template,
    enabled: check.enabled,
  };
}

interface CustomCheckDialogProps {
  open: boolean;
  editing: CustomCheck | null;
  saving: boolean;
  onOpenChange: (open: boolean) => void;
  onSave: (params: CustomCheckParams) => void;
  onValidationError: (message: string) => void;
}

export function CustomCheckDialog({
  open,
  editing,
  saving,
  onOpenChange,
  onSave,
  onValidationError,
}: CustomCheckDialogProps) {
  const [form, setForm] = useFormSync(open, editing, EMPTY_PARAMS, paramsFrom);
  const { tags } = useCheckFieldTags();

  const needsThreshold = form.operator !== "missing";

  function handleSave() {
    if (!form.name.trim() || !form.field.trim() || !form.message_template.trim()) {
      onValidationError("Name, field, and message template are required");
      return;
    }
    onSave(form);
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
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
                value={form.severity as CheckSeverity}
                onValueChange={(v) => setForm((f) => ({ ...f, severity: v as CheckSeverity }))}
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
                value={form.operator as CheckOperator}
                onValueChange={(v) =>
                  setForm((f) => ({ ...f, operator: v as CheckOperator, threshold: null }))
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
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
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
  );
}
