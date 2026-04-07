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
  listCustomExtractors,
  createCustomExtractor,
  updateCustomExtractor,
  deleteCustomExtractor,
  type CustomExtractor,
  type CustomExtractorParams,
} from "@/src/api/extension";
import { ExtractorDialog } from "./ExtractorDialog";
import { ExtractorRow } from "./ExtractorRow";

export function ExtractorsSettings() {
  const { data: extractors = [], mutate } = useSWR("custom-extractors", listCustomExtractors);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editing, setEditing] = useState<CustomExtractor | null>(null);
  const [saving, setSaving] = useState(false);

  function openCreate() {
    setEditing(null);
    setDialogOpen(true);
  }

  function openEdit(extractor: CustomExtractor) {
    setEditing(extractor);
    setDialogOpen(true);
  }

  async function handleSave(form: CustomExtractorParams) {
    setSaving(true);
    try {
      if (editing) {
        const updated = await updateCustomExtractor(editing.id, form);
        mutate((prev = []) => prev.map((e) => (e.id === editing.id ? updated : e)), false);
        toast.success("Extractor updated");
      } else {
        const created = await createCustomExtractor(form);
        mutate((prev = []) => [...prev, created], false);
        toast.success("Extractor created");
      }
      setDialogOpen(false);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Failed to save extractor");
    } finally {
      setSaving(false);
    }
  }

  async function handleDelete(id: string) {
    try {
      await deleteCustomExtractor(id);
      mutate((prev = []) => prev.filter((e) => e.id !== id), false);
      toast.success("Extractor deleted");
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Failed to delete extractor");
    }
  }

  async function handleToggleEnabled(extractor: CustomExtractor) {
    try {
      const updated = await updateCustomExtractor(extractor.id, {
        ...extractor,
        attribute: extractor.attribute ?? null,
        enabled: !extractor.enabled,
      });
      mutate((prev = []) => prev.map((e) => (e.id === extractor.id ? updated : e)), false);
    } catch {
      toast.error("Failed to toggle extractor");
    }
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <p className="text-sm text-muted-foreground">
          Extractors pull custom data from every crawled page using CSS selectors.
        </p>
        <Button size="sm" onClick={openCreate}>
          <Plus className="h-4 w-4 mr-1" />
          Add Extractor
        </Button>
      </div>

      {extractors.length === 0 ? (
        <p className="text-sm text-muted-foreground py-8 text-center">
          No extractors configured.
        </p>
      ) : (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Name</TableHead>
              <TableHead>Key</TableHead>
              <TableHead>Selector</TableHead>
              <TableHead>Mode</TableHead>
              <TableHead>Enabled</TableHead>
              <TableHead className="w-20" />
            </TableRow>
          </TableHeader>
          <TableBody>
            {extractors.map((e) => (
              <ExtractorRow
                key={e.id}
                extractor={e}
                onEdit={openEdit}
                onDelete={handleDelete}
                onToggleEnabled={handleToggleEnabled}
              />
            ))}
          </TableBody>
        </Table>
      )}

      <ExtractorDialog
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
