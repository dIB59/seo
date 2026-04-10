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
  listCustomChecks,
  createCustomCheck,
  updateCustomCheck,
  deleteCustomCheck,
  type CustomCheck,
  type CustomCheckParams,
} from "@/src/api/extension";
import { useCrudState } from "@/src/hooks/use-crud-state";
import { CustomCheckDialog } from "./CustomCheckDialog";
import { CustomCheckRow } from "./CustomCheckRow";

export function CustomChecksSettings() {
  const crud = useCrudState<CustomCheck, CustomCheckParams>({
    swrKey: "custom-checks",
    fetcher: listCustomChecks,
    onCreate: createCustomCheck,
    onUpdate: updateCustomCheck,
    onDelete: deleteCustomCheck,
    entityName: "Check",
  });

  async function handleToggleEnabled(check: CustomCheck) {
    try {
      const updated = await updateCustomCheck(check.id, {
        ...check,
        threshold: check.threshold ?? null,
        enabled: !check.enabled,
      });
      crud.mutate(
        (prev = []) => prev.map((c) => (c.id === check.id ? updated : c)),
        false,
      );
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
        <Button size="sm" onClick={crud.openCreate}>
          <Plus className="h-4 w-4 mr-1" />
          Add Check
        </Button>
      </div>

      {crud.items.length === 0 ? (
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
            {crud.items.map((c) => (
              <CustomCheckRow
                key={c.id}
                check={c}
                onEdit={crud.openEdit}
                onDelete={crud.handleDelete}
                onToggleEnabled={handleToggleEnabled}
              />
            ))}
          </TableBody>
        </Table>
      )}

      <CustomCheckDialog
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
