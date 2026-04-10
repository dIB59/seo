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
  listReportPatterns,
  createReportPattern,
  updateReportPattern,
  toggleReportPattern,
  deleteReportPattern,
  type ReportPattern,
  type ReportPatternParams,
} from "@/src/api/report";
import { useCrudState } from "@/src/hooks/use-crud-state";
import { ReportPatternDialog } from "./ReportPatternDialog";
import { ReportPatternRow } from "./ReportPatternRow";

export function ReportPatternsSettings() {
  const crud = useCrudState<ReportPattern, ReportPatternParams>({
    swrKey: "report-patterns",
    fetcher: listReportPatterns,
    onCreate: createReportPattern,
    onUpdate: updateReportPattern,
    onDelete: deleteReportPattern,
    entityName: "Pattern",
  });

  async function handleToggle(p: ReportPattern) {
    try {
      await toggleReportPattern(p.id, !p.enabled);
      crud.mutate(
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
        <Button size="sm" onClick={crud.openCreate}>
          <Plus className="h-4 w-4 mr-1" />
          Add Pattern
        </Button>
      </div>

      {crud.items.length === 0 ? (
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
            {crud.items.map((p) => (
              <ReportPatternRow
                key={p.id}
                pattern={p}
                onEdit={crud.openEdit}
                onDelete={crud.handleDelete}
                onToggle={handleToggle}
              />
            ))}
          </TableBody>
        </Table>
      )}

      <ReportPatternDialog
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
