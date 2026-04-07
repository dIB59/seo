"use client";

import { Lock, Pencil, Trash2 } from "lucide-react";

import { Badge } from "@/src/components/ui/badge";
import { Button } from "@/src/components/ui/button";
import { Switch } from "@/src/components/ui/switch";
import { TableCell, TableRow } from "@/src/components/ui/table";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/src/components/ui/tooltip";

import type { ReportPattern } from "@/src/api/report";
import type { PatternSeverity } from "@/src/bindings";
import { OPERATOR_OPTIONS } from "./report-pattern-options";

const SEVERITY_BADGE: Record<PatternSeverity, "destructive" | "secondary" | "outline"> = {
  critical: "destructive",
  warning: "secondary",
  suggestion: "outline",
};

interface ReportPatternRowProps {
  pattern: ReportPattern;
  onEdit: (p: ReportPattern) => void;
  onDelete: (id: string) => void;
  onToggle: (p: ReportPattern) => void;
}

export function ReportPatternRow({ pattern, onEdit, onDelete, onToggle }: ReportPatternRowProps) {
  const operatorLabel = OPERATOR_OPTIONS.find((o) => o.value === pattern.operator)?.label;

  return (
    <TableRow className={!pattern.enabled ? "opacity-50" : undefined}>
      <TableCell className="font-medium">
        <div className="flex items-center gap-1.5">
          {pattern.isBuiltin && (
            <Tooltip>
              <TooltipTrigger asChild>
                <Lock className="h-3 w-3 text-muted-foreground shrink-0" />
              </TooltipTrigger>
              <TooltipContent>Built-in pattern</TooltipContent>
            </Tooltip>
          )}
          <span>{pattern.name}</span>
        </div>
      </TableCell>
      <TableCell>
        <Badge variant="outline" className="text-xs capitalize">
          {pattern.category}
        </Badge>
      </TableCell>
      <TableCell>
        <Badge variant={SEVERITY_BADGE[pattern.severity]} className="text-xs capitalize">
          {pattern.severity}
        </Badge>
      </TableCell>
      <TableCell className="text-xs text-muted-foreground">
        <code>{pattern.field}</code> {operatorLabel}
        {pattern.threshold && (
          <>
            {" "}
            <code>{pattern.threshold}</code>
          </>
        )}
      </TableCell>
      <TableCell className="text-xs text-muted-foreground">
        {(pattern.minPrevalence * 100).toFixed(0)}%
      </TableCell>
      <TableCell>
        <Switch checked={pattern.enabled} onCheckedChange={() => onToggle(pattern)} />
      </TableCell>
      <TableCell>
        <div className="flex gap-1">
          {!pattern.isBuiltin && (
            <>
              <Button
                variant="ghost"
                size="icon"
                className="h-7 w-7"
                aria-label="Edit pattern"
                onClick={() => onEdit(pattern)}
              >
                <Pencil className="h-3.5 w-3.5" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                className="h-7 w-7 text-destructive hover:text-destructive"
                aria-label="Delete pattern"
                onClick={() => onDelete(pattern.id)}
              >
                <Trash2 className="h-3.5 w-3.5" />
              </Button>
            </>
          )}
        </div>
      </TableCell>
    </TableRow>
  );
}
