"use client";

import { Pencil, Trash2 } from "lucide-react";

import { Badge } from "@/src/components/ui/badge";
import { Button } from "@/src/components/ui/button";
import { Switch } from "@/src/components/ui/switch";
import { TableCell, TableRow } from "@/src/components/ui/table";

import type { CustomCheck } from "@/src/api/extension";
import { OPERATOR_OPTIONS, type CheckSeverity } from "./custom-check-options";

const SEVERITY_VARIANT: Record<CheckSeverity, "destructive" | "secondary" | "outline"> = {
  critical: "destructive",
  warning: "secondary",
  info: "outline",
};

interface CustomCheckRowProps {
  check: CustomCheck;
  onEdit: (c: CustomCheck) => void;
  onDelete: (id: string) => void;
  onToggleEnabled: (c: CustomCheck) => void;
}

export function CustomCheckRow({
  check,
  onEdit,
  onDelete,
  onToggleEnabled,
}: CustomCheckRowProps) {
  const operatorLabel = OPERATOR_OPTIONS.find((o) => o.value === check.operator)?.label;

  return (
    <TableRow>
      <TableCell className="font-medium">{check.name}</TableCell>
      <TableCell>
        <Badge
          variant={SEVERITY_VARIANT[check.severity as CheckSeverity]}
          className="text-xs capitalize"
        >
          {check.severity}
        </Badge>
      </TableCell>
      <TableCell className="text-sm text-muted-foreground">
        <code className="text-xs">{check.field}</code> {operatorLabel}
        {check.threshold && <> {check.threshold}</>}
      </TableCell>
      <TableCell>
        <Switch checked={check.enabled} onCheckedChange={() => onToggleEnabled(check)} />
      </TableCell>
      <TableCell>
        <div className="flex gap-1">
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            aria-label="Edit check"
            onClick={() => onEdit(check)}
          >
            <Pencil className="h-3.5 w-3.5" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7 text-destructive hover:text-destructive"
            aria-label="Delete check"
            onClick={() => onDelete(check.id)}
          >
            <Trash2 className="h-3.5 w-3.5" />
          </Button>
        </div>
      </TableCell>
    </TableRow>
  );
}
