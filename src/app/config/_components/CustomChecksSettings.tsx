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
  listCustomChecks,
  createCustomCheck,
  updateCustomCheck,
  deleteCustomCheck,
  type CustomCheck,
  type CustomCheckParams,
} from "@/src/api/extension";
import { CustomCheckDialog } from "./CustomCheckDialog";
import { CustomCheckRow } from "./CustomCheckRow";

export function CustomChecksSettings() {
  const { data: checks = [], mutate } = useSWR("custom-checks", listCustomChecks);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editing, setEditing] = useState<CustomCheck | null>(null);
  const [saving, setSaving] = useState(false);

  function openCreate() {
    setEditing(null);
    setDialogOpen(true);
  }

  function openEdit(check: CustomCheck) {
    setEditing(check);
    setDialogOpen(true);
  }

  async function handleSave(form: CustomCheckParams) {
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
        <p className="text-sm text-muted-foreground py-8 text-center">
          No custom checks configured.
        </p>
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
              <CustomCheckRow
                key={c.id}
                check={c}
                onEdit={openEdit}
                onDelete={handleDelete}
                onToggleEnabled={handleToggleEnabled}
              />
            ))}
          </TableBody>
        </Table>
      )}

      <CustomCheckDialog
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
