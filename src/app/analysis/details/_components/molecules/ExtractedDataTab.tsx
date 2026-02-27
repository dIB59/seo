"use client";

import { useMemo } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/src/components/ui/card";
import { Badge } from "@/src/components/ui/badge";

interface ExtractedDataTabProps {
  data: Record<string, unknown>;
}

/**
 * Automatically displays extracted data from custom extractors.
 * This component dynamically renders all key-value pairs from the extracted data,
 * so it works with ANY extractor type without requiring additional code changes.
 */
export default function ExtractedDataTab({ data }: ExtractedDataTabProps) {
  const entries = useMemo(() => Object.entries(data), [data]);

  if (entries.length === 0) {
    return (
      <div className="text-center py-12 text-muted-foreground">
        <p className="text-lg font-medium">No extracted data</p>
        <p className="text-sm mt-2">Data from custom extractors will appear here when available.</p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold">Custom Extractor Data</h3>
        <Badge variant="secondary">{entries.length} fields</Badge>
      </div>

      <div className="grid gap-4">
        {entries.map(([key, value]) => (
          <Card key={key}>
            <CardHeader className="py-3">
              <CardTitle className="text-sm font-mono">{key}</CardTitle>
            </CardHeader>
            <CardContent className="py-2">
              <RenderedValue value={value} />
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}

/**
 * Renders a value based on its type
 */
function RenderedValue({ value }: { value: unknown }) {
  if (value === null || value === undefined) {
    return <span className="text-muted-foreground italic">null</span>;
  }

  if (typeof value === "string") {
    return <span className="text-sm break-all">{value}</span>;
  }

  if (typeof value === "number" || typeof value === "boolean") {
    return <span className="text-sm font-mono">{String(value)}</span>;
  }

  if (Array.isArray(value)) {
    if (value.length === 0) {
      return <span className="text-muted-foreground italic">empty array</span>;
    }
    return (
      <ul className="space-y-1">
        {value.map((item, index) => (
          <li key={index} className="text-sm">
            <RenderedValue value={item} />
          </li>
        ))}
      </ul>
    );
  }

  if (typeof value === "object") {
    const entries = Object.entries(value as Record<string, unknown>);
    if (entries.length === 0) {
      return <span className="text-muted-foreground italic">empty object</span>;
    }
    return (
      <div className="space-y-2 pl-4 border-l-2 border-muted">
        {entries.map(([k, v]) => (
          <div key={k} className="grid grid-cols-[150px_1fr] gap-2">
            <span className="text-xs font-mono text-muted-foreground">{k}:</span>
            <RenderedValue value={v} />
          </div>
        ))}
      </div>
    );
  }

  return <span className="text-sm">{String(value)}</span>;
}
