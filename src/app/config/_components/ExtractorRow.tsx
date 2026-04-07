"use client";

import { Pencil, Trash2 } from "lucide-react";

import { Badge } from "@/src/components/ui/badge";
import { Button } from "@/src/components/ui/button";
import { Switch } from "@/src/components/ui/switch";
import { TableCell, TableRow } from "@/src/components/ui/table";

import type { CustomExtractor } from "@/src/api/extension";

interface ExtractorRowProps {
  extractor: CustomExtractor;
  onEdit: (e: CustomExtractor) => void;
  onDelete: (id: string) => void;
  onToggleEnabled: (e: CustomExtractor) => void;
}

export function ExtractorRow({ extractor, onEdit, onDelete, onToggleEnabled }: ExtractorRowProps) {
  return (
    <TableRow>
      <TableCell className="font-medium">{extractor.name}</TableCell>
      <TableCell>
        <code className="text-xs bg-muted px-1 py-0.5 rounded">{extractor.key}</code>
      </TableCell>
      <TableCell>
        <code className="text-xs bg-muted px-1 py-0.5 rounded">{extractor.selector}</code>
        {extractor.attribute && (
          <Badge variant="outline" className="ml-1 text-xs">
            @{extractor.attribute}
          </Badge>
        )}
      </TableCell>
      <TableCell>
        <Badge variant={extractor.multiple ? "secondary" : "outline"} className="text-xs">
          {extractor.multiple ? "all" : "first"}
        </Badge>
      </TableCell>
      <TableCell>
        <Switch checked={extractor.enabled} onCheckedChange={() => onToggleEnabled(extractor)} />
      </TableCell>
      <TableCell>
        <div className="flex gap-1">
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            aria-label="Edit extractor"
            onClick={() => onEdit(extractor)}
          >
            <Pencil className="h-3.5 w-3.5" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7 text-destructive hover:text-destructive"
            aria-label="Delete extractor"
            onClick={() => onDelete(extractor.id)}
          >
            <Trash2 className="h-3.5 w-3.5" />
          </Button>
        </div>
      </TableCell>
    </TableRow>
  );
}
