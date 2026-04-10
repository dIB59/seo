"use client";

import useSWR from "swr";
import { Badge } from "@/src/components/ui/badge";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/src/components/ui/table";
import { listTags, type Tag } from "@/src/api/extension";

function sourceLabel(tag: Tag): string {
  if (tag.source.kind === "extractor") return tag.source.extractorName;
  return "Built-in";
}

function dataTypeBadge(dt: string) {
  const colors: Record<string, string> = {
    text: "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
    number: "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200",
    bool: "bg-amber-100 text-amber-800 dark:bg-amber-900 dark:text-amber-200",
    list: "bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200",
  };
  return (
    <span
      className={`inline-block text-[10px] font-medium px-1.5 py-0.5 rounded ${colors[dt] ?? ""}`}
    >
      {dt}
    </span>
  );
}

function scopeBadges(scopes: string[]) {
  const labels: Record<string, string> = {
    checkField: "Check",
    checkMessage: "Message",
    templateText: "Template",
    templateCondition: "Condition",
    aiPrompt: "AI Prompt",
  };
  return (
    <div className="flex flex-wrap gap-1">
      {scopes.map((s) => (
        <Badge key={s} variant="outline" className="text-[10px] py-0">
          {labels[s] ?? s}
        </Badge>
      ))}
    </div>
  );
}

export function TagsSettings() {
  const { data: tags = [] } = useSWR("tags", () => listTags());

  const builtinTags = tags.filter((t) => t.source.kind === "builtin");
  const extractorTags = tags.filter((t) => t.source.kind === "extractor");

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold mb-1">Tags</h3>
        <p className="text-sm text-muted-foreground mb-4">
          Tags are symbols you can reference in custom checks, AI prompts, and
          report templates. Built-in tags come from page data. Extractor tags
          come from your custom extractors.
        </p>
      </div>

      {extractorTags.length > 0 && (
        <div>
          <h4 className="text-sm font-medium mb-2">Your Extractor Tags</h4>
          <p className="text-xs text-muted-foreground mb-3">
            Reference these as <code className="text-[10px]">tag:name</code> in
            check fields, or as{" "}
            <code className="text-[10px]">{"{tag.name}"}</code> in prompts and
            templates.
          </p>
          <TagTable tags={extractorTags} showSource />
        </div>
      )}

      <div>
        <h4 className="text-sm font-medium mb-2">Built-in Tags</h4>
        <TagTable tags={builtinTags} showSource={false} />
      </div>
    </div>
  );
}

function TagTable({ tags, showSource }: { tags: Tag[]; showSource: boolean }) {
  return (
    <div className="rounded-md border">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead className="w-[180px]">Name</TableHead>
            <TableHead className="w-[180px]">Label</TableHead>
            {showSource && <TableHead className="w-[140px]">Source</TableHead>}
            <TableHead className="w-[70px]">Type</TableHead>
            <TableHead>Description</TableHead>
            <TableHead className="w-[200px]">Scopes</TableHead>
            <TableHead className="w-[120px]">Example</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {tags.map((tag) => (
            <TableRow key={tag.name}>
              <TableCell>
                <code className="text-xs bg-muted px-1 py-0.5 rounded">
                  {tag.name}
                </code>
              </TableCell>
              <TableCell className="text-sm">{tag.label}</TableCell>
              {showSource && (
                <TableCell className="text-xs text-muted-foreground">
                  {sourceLabel(tag)}
                </TableCell>
              )}
              <TableCell>{dataTypeBadge(tag.dataType)}</TableCell>
              <TableCell className="text-xs text-muted-foreground">
                {tag.description}
              </TableCell>
              <TableCell>{scopeBadges(tag.scopes)}</TableCell>
              <TableCell>
                {tag.example && (
                  <code className="text-[10px] text-muted-foreground">
                    {tag.example}
                  </code>
                )}
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}
