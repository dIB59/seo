"use client";

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
import { useCrudState } from "@/src/hooks/use-crud-state";
import { ExtractorDialog } from "./ExtractorDialog";
import { ExtractorRow } from "./ExtractorRow";

export function ExtractorsSettings() {
  const crud = useCrudState<CustomExtractor, CustomExtractorParams>({
    swrKey: "custom-extractors",
    fetcher: listCustomExtractors,
    onCreate: createCustomExtractor,
    onUpdate: updateCustomExtractor,
    onDelete: deleteCustomExtractor,
    entityName: "Extractor",
  });

  async function handleToggleEnabled(extractor: CustomExtractor) {
    try {
      const updated = await updateCustomExtractor(extractor.id, {
        ...extractor,
        attribute: extractor.attribute ?? null,
        enabled: !extractor.enabled,
      });
      crud.mutate(
        (prev = []) => prev.map((e) => (e.id === extractor.id ? updated : e)),
        false,
      );
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
        <Button size="sm" onClick={crud.openCreate}>
          <Plus className="h-4 w-4 mr-1" />
          Add Extractor
        </Button>
      </div>

      {crud.items.length === 0 ? (
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
            {crud.items.map((e) => (
              <ExtractorRow
                key={e.id}
                extractor={e}
                onEdit={crud.openEdit}
                onDelete={crud.handleDelete}
                onToggleEnabled={handleToggleEnabled}
              />
            ))}
          </TableBody>
        </Table>
      )}

      <ExtractorDialog
        open={crud.dialogOpen}
        editing={crud.editing}
        saving={crud.saving}
        onOpenChange={crud.setDialogOpen}
        onSave={crud.handleSave}
        onValidationError={(msg) => toast.error(msg)}
      />
    </div>
  );
}
