"use client";

import { useState } from "react";
import useSWR from "swr";
import { Plus } from "lucide-react";
import { toast } from "sonner";

import { Button } from "@/src/components/ui/button";
import {
  Table,
  TableBody,
  TableHead,
  TableHeader,
  TableRow,
} from "@/src/components/ui/table";

import {
  listReportPatterns,
  createReportPattern,
  updateReportPattern,
  toggleReportPattern,
  deleteReportPattern,
  type ReportPattern,
  type ReportPatternParams,
} from "@/src/api/report";
import { ReportPatternDialog } from "./ReportPatternDialog";
import { ReportPatternRow } from "./ReportPatternRow";

export function ReportPatternsSettings() {
  const { data: patterns = [], mutate } = useSWR("report-patterns", listReportPatterns);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editing, setEditing] = useState<ReportPattern | null>(null);
  const [saving, setSaving] = useState(false);

  function openCreate() {
    setEditing(null);
    setDialogOpen(true);
  }

  function openEdit(p: ReportPattern) {
    setEditing(p);
    setDialogOpen(true);
  }

  async function handleSave(form: ReportPatternParams) {
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
      mutate(
        (prev = []) => prev.map((x) => (x.id === p.id ? { ...x, enabled: !x.enabled } : x)),
        false,
      );
    } catch {
      toast.error("Failed to toggle pattern");
    }
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <p className="text-sm text-muted-foreground">
          Patterns are evaluated site-wide when generating a report. Built-in patterns can be
          disabled but not deleted. Custom patterns integrate with your extractors and custom checks.
        </p>
        <Button size="sm" onClick={openCreate}>
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
              <ReportPatternRow
                key={p.id}
                pattern={p}
                onEdit={openEdit}
                onDelete={handleDelete}
                onToggle={handleToggle}
              />
            ))}
          </TableBody>
        </Table>
      )}

      <ReportPatternDialog
        open={dialogOpen}
        editing={editing}
        saving={saving}
        onOpenChange={setDialogOpen}
        onSave={handleSave}
        onValidationError={(msg) => toast.error(msg)}
      />
    </div>
  );
}
