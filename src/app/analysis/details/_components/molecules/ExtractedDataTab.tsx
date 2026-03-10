"use client";

import { useMemo } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/src/components/ui/card";
import { Badge } from "@/src/components/ui/badge";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/src/components/ui/table";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/src/components/ui/collapsible";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/src/components/ui/tooltip";
import { Database, ChevronDown, Key } from "lucide-react";

interface ExtractedDataTabProps {
  data: Record<string, unknown>;
}

function formatFieldLabel(key: string): string {
  return key
    .replace(/^[^.]+\./, "")
    .replace(/[_.:-]+/g, " ")
    .replace(/\b\w/g, (match) => match.toUpperCase());
}

/**
 * Determine the display type for a value
 */
function getValueType(
  value: unknown,
): "string" | "number" | "boolean" | "array" | "object" | "null" {
  if (value === null || value === undefined) return "null";
  if (Array.isArray(value)) return "array";
  if (typeof value === "object") return "object";
  if (typeof value === "number") return "number";
  if (typeof value === "boolean") return "boolean";
  return "string";
}

/**
 * Check if an object looks like a structured data item (has @type, @context, etc.)
 */
function isStructuredData(obj: Record<string, unknown>): boolean {
  return "@context" in obj || "@type" in obj;
}

/**
 * Check if array contains objects (for table rendering)
 */
function isArrayOfObjects(arr: unknown[]): boolean {
  return arr.length > 0 && typeof arr[0] === "object" && arr[0] !== null && !Array.isArray(arr[0]);
}

/**
 * Get common keys from an array of objects for table headers
 */
function getCommonKeys(arr: Record<string, unknown>[]): string[] {
  if (arr.length === 0) return [];
  const keyCount = new Map<string, number>();

  arr.forEach((item) => {
    Object.keys(item).forEach((key) => {
      keyCount.set(key, (keyCount.get(key) || 0) + 1);
    });
  });

  // Return keys that appear in at least 50% of items
  const threshold = Math.ceil(arr.length * 0.5);
  return Array.from(keyCount.entries())
    .filter(([, count]) => count >= threshold)
    .sort((a, b) => b[1] - a[1])
    .map(([key]) => key);
}

function HoverableText({
  value,
  href,
  className,
}: {
  value: string;
  href?: string;
  className?: string;
}) {
  const sharedClassName = "text-sm break-all whitespace-normal inline-block max-w-full align-top";

  if (href) {
    return (
      <Tooltip>
        <TooltipTrigger asChild>
          <a
            href={href}
            target="_blank"
            rel="noopener noreferrer"
            className={`text-primary hover:underline ${sharedClassName} ${className ?? ""}`}
          >
            {value}
          </a>
        </TooltipTrigger>
        <TooltipContent className="max-w-md">
          <p className="break-words">{value}</p>
        </TooltipContent>
      </Tooltip>
    );
  }

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <span className={`${sharedClassName} cursor-default ${className ?? ""}`}>{value}</span>
      </TooltipTrigger>
      <TooltipContent className="max-w-md">
        <p className="break-words">{value}</p>
      </TooltipContent>
    </Tooltip>
  );
}

/**
 * Renders a value based on its type with collapsible support for nested objects
 */
function RenderedValue({ value, depth = 0 }: { value: unknown; depth?: number }) {
  const type = getValueType(value);

  if (type === "null") {
    return <span className="text-muted-foreground italic">null</span>;
  }

  if (type === "string") {
    // Check if it looks like a URL
    const strValue = value as string;
    if (strValue.startsWith("http://") || strValue.startsWith("https://")) {
      return <HoverableText value={strValue} href={strValue} />;
    }
    return <HoverableText value={strValue} />;
  }

  if (type === "number" || type === "boolean") {
    return <span className="text-sm font-mono">{String(value)}</span>;
  }

  if (type === "array") {
    const arr = value as unknown[];
    if (arr.length === 0) {
      return <span className="text-muted-foreground italic">empty array</span>;
    }

    // Check if array of primitives or objects
    if (arr.every((item) => typeof item !== "object" || item === null)) {
      // Array of primitives
      return (
        <div className="flex flex-wrap gap-1">
          {arr.map((item, idx) => (
            <Badge
              key={idx}
              variant="secondary"
              className="text-xs max-w-full break-all whitespace-normal"
            >
              {String(item)}
            </Badge>
          ))}
        </div>
      );
    }

    // Array of objects - render as table
    if (isArrayOfObjects(arr)) {
      const objects = arr as Record<string, unknown>[];
      const keys = getCommonKeys(objects);

      if (keys.length > 0 && depth < 2) {
        return (
          <div className="overflow-x-auto rounded-md border bg-card/40">
            <Table className="table-fixed min-w-[840px]">
              <TableHeader>
                <TableRow>
                  {keys.map((key) => (
                    <TableHead
                      key={key}
                      className="text-xs whitespace-normal break-words align-top min-w-[140px]"
                    >
                      {key}
                    </TableHead>
                  ))}
                </TableRow>
              </TableHeader>
              <TableBody>
                {objects.map((obj, idx) => (
                  <TableRow key={idx}>
                    {keys.map((key) => (
                      <TableCell
                        key={key}
                        className="text-xs max-w-[320px] align-top whitespace-normal break-all min-w-0"
                      >
                        <div className="min-w-0">
                          <RenderedValue value={obj[key]} depth={depth + 1} />
                        </div>
                      </TableCell>
                    ))}
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>
        );
      }
    }

    // Fallback to list
    return (
      <ul className="space-y-1">
        {arr.map((item, idx) => (
          <li key={idx} className="text-sm">
            <RenderedValue value={item} depth={depth + 1} />
          </li>
        ))}
      </ul>
    );
  }

  if (type === "object") {
    const obj = value as Record<string, unknown>;
    const entries = Object.entries(obj);

    if (entries.length === 0) {
      return <span className="text-muted-foreground italic">empty object</span>;
    }

    // Check if it's structured data
    if (isStructuredData(obj)) {
      return (
        <div className="space-y-2">
          {entries.map(([k, v]) => (
            <div key={k} className="grid grid-cols-[120px_1fr] gap-2">
              <span className="text-xs font-mono text-muted-foreground">{k}:</span>
              <RenderedValue value={v} depth={depth + 1} />
            </div>
          ))}
        </div>
      );
    }

    // For nested objects with depth > 1, use collapsible
    if (depth < 1) {
      return (
        <Collapsible>
          <CollapsibleTrigger className="flex items-center gap-1 text-sm text-primary hover:underline">
            <ChevronDown className="h-3 w-3" />
            {entries.length} properties
          </CollapsibleTrigger>
          <CollapsibleContent>
            <div className="space-y-2 mt-2 pl-4 border-l-2 border-muted min-w-0">
              {entries.map(([k, v]) => (
                <div key={k} className="grid grid-cols-[150px_1fr] gap-2 min-w-0">
                  <span className="text-xs font-mono text-muted-foreground break-all">{k}:</span>
                  <RenderedValue value={v} depth={depth + 1} />
                </div>
              ))}
            </div>
          </CollapsibleContent>
        </Collapsible>
      );
    }

    return (
      <div className="space-y-2 pl-4 border-l-2 border-muted min-w-0">
        {entries.map(([k, v]) => (
          <div key={k} className="grid grid-cols-[150px_1fr] gap-2 min-w-0">
            <span className="text-xs font-mono text-muted-foreground break-all">{k}:</span>
            <RenderedValue value={v} depth={depth + 1} />
          </div>
        ))}
      </div>
    );
  }

  return <span className="text-sm">{String(value)}</span>;
}

/**
 * Render a single key-value pair as a card
 */
function KeyValueCard({ dataKey, value }: { dataKey: string; value: unknown }) {
  const label = formatFieldLabel(dataKey);
  const type = getValueType(value);

  return (
    <Card>
      <CardHeader className="py-3">
        <CardTitle className="text-sm font-medium flex items-center gap-2">
          <Key className="h-4 w-4 text-muted-foreground" />
          {label}
          <Badge variant="outline" className="ml-auto text-xs font-mono">
            {type}
          </Badge>
        </CardTitle>
      </CardHeader>
      <CardContent className="py-2">
        <RenderedValue value={value} />
      </CardContent>
    </Card>
  );
}

/**
 * Main ExtractedDataTab component that displays extracted data grouped by category
 */
export default function ExtractedDataTab({ data }: ExtractedDataTabProps) {
  const entries = useMemo(() => {
    if (!data) return [];
    return Object.entries(data);
  }, [data]);

  if (entries.length === 0) {
    return (
      <Card>
        <CardContent className="py-12 text-center">
          <Database className="h-12 w-12 text-muted-foreground mx-auto mb-3" />
          <p className="text-lg font-medium">No extracted data</p>
          <p className="text-sm text-muted-foreground mt-2">
            Data from custom extractors will appear here when available.
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <TooltipProvider>
      <div className="space-y-3">
        {entries.map(([key, value]) => (
          <KeyValueCard key={key} dataKey={key} value={value} />
        ))}
      </div>
    </TooltipProvider>
  );
}
