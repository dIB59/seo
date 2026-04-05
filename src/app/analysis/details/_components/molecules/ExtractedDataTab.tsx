import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from "@/src/components/ui/table";
import { Badge } from "@/src/components/ui/badge";
import { Card, CardContent } from "@/src/components/ui/card";
import {
    Tooltip,
    TooltipContent,
    TooltipProvider,
    TooltipTrigger,
} from "@/src/components/ui/tooltip";
import { Code2, ExternalLink } from "lucide-react";
import CharLengthBadge from "@/src/app/analysis/details/_components/atoms/CharLengthBadge";
import type { JsonValue } from "@/src/bindings";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function formatKey(key: string): string {
    return key.replace(/[_.:-]+/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
}

function isUrl(s: string): boolean {
    return s.startsWith("http://") || s.startsWith("https://");
}

// ---------------------------------------------------------------------------
// Single string value cell
// ---------------------------------------------------------------------------

function StringValue({ value }: { value: string }) {
    if (isUrl(value)) {
        return (
            <Tooltip>
                <TooltipTrigger asChild>
                    <a
                        href={value}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="inline-flex items-center gap-1 text-sm text-primary hover:underline break-all"
                    >
                        {value}
                        <ExternalLink className="h-3 w-3 shrink-0" />
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
                <span className="text-sm break-all whitespace-normal cursor-default">{value}</span>
            </TooltipTrigger>
            <TooltipContent className="max-w-md">
                <p className="break-words">{value}</p>
            </TooltipContent>
        </Tooltip>
    );
}

// ---------------------------------------------------------------------------
// Array value — shown as a stack of chips
// ---------------------------------------------------------------------------

function ArrayValue({ items }: { items: string[] }) {
    return (
        <div className="flex flex-wrap gap-1.5 py-0.5">
            {items.map((item, i) => {
                if (isUrl(item)) {
                    return (
                        <a
                            key={i}
                            href={item}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="inline-flex items-center gap-1 text-xs bg-muted border border-border/60 rounded px-2 py-0.5 text-primary hover:bg-muted/80 hover:underline max-w-full truncate"
                            title={item}
                        >
                            {item}
                            <ExternalLink className="h-2.5 w-2.5 shrink-0" />
                        </a>
                    );
                }
                return (
                    <code
                        key={i}
                        className="text-xs bg-muted border border-border/60 rounded px-2 py-0.5 text-foreground max-w-full truncate"
                        title={item}
                    >
                        {item}
                    </code>
                );
            })}
        </div>
    );
}

// ---------------------------------------------------------------------------
// Length / count cell
// ---------------------------------------------------------------------------

function LengthCell({ value }: { value: JsonValue }) {
    if (typeof value === "string" && value.length > 0) {
        return <CharLengthBadge length={value.length} />;
    }
    if (Array.isArray(value)) {
        return (
            <Badge variant="outline" className="text-xs font-mono bg-muted">
                {value.length} item{value.length !== 1 ? "s" : ""}
            </Badge>
        );
    }
    return <span className="text-muted-foreground">-</span>;
}

// ---------------------------------------------------------------------------
// Content cell — dispatches on value type
// ---------------------------------------------------------------------------

function ContentCell({ value }: { value: JsonValue }) {
    // Array of strings
    if (Array.isArray(value)) {
        const strings = value
            .map((v) => (typeof v === "string" ? v : JSON.stringify(v)))
            .filter(Boolean);

        if (strings.length === 0) {
            return <span className="text-muted-foreground italic text-sm">Empty</span>;
        }
        return <ArrayValue items={strings} />;
    }

    // Single string
    if (typeof value === "string") {
        return value.trim() ? (
            <StringValue value={value} />
        ) : (
            <span className="text-muted-foreground italic text-sm">Empty</span>
        );
    }

    // Number / boolean / object — show as code
    const raw = JSON.stringify(value, null, 2);
    return (
        <pre className="text-xs bg-muted rounded p-2 whitespace-pre-wrap break-all max-h-32 overflow-auto">
            {raw}
        </pre>
    );
}

// ---------------------------------------------------------------------------
// Main tab
// ---------------------------------------------------------------------------

interface Props {
    extractedData: Partial<Record<string, JsonValue>>;
}

export default function ExtractedDataTab({ extractedData }: Props) {
    console.log("extractedData");
    console.log(extractedData);
    const rows = Object.entries(extractedData)
        .filter(([, value]) => {
            if (value === null || value === undefined) return false;
            if (typeof value === "string") return value.trim().length > 0;
            if (Array.isArray(value)) return value.length > 0;
            return true;
        })
        .sort(([a], [b]) => a.localeCompare(b));

    if (rows.length === 0) {
        return (
            <Card>
                <CardContent className="py-16 text-center text-sm text-muted-foreground">
                    No extracted data for this page.
                    <br />
                    <span className="text-xs">
                        Add extractors in Settings → Custom Extractors and re-crawl the site.
                    </span>
                </CardContent>
            </Card>
        );
    }

    return (
        <TooltipProvider>
            <Card>
                <CardContent className="pt-6">
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead className="w-50">Field</TableHead>
                                <TableHead>Content</TableHead>
                                <TableHead className="w-[110px] text-right">Size</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {rows.map(([key, value]) => (
                                <TableRow key={key}>
                                    <TableCell className="font-medium align-top pt-3">
                                        <div className="flex items-center gap-2">
                                            <Code2 className="h-4 w-4 text-muted-foreground shrink-0" />
                                            <span>{formatKey(key)}</span>
                                        </div>
                                        <code className="text-[10px] text-muted-foreground/60 ml-6 font-mono">
                                            {key}
                                        </code>
                                    </TableCell>
                                    <TableCell className="align-top pt-3 whitespace-normal">
                                        <ContentCell value={value!} />
                                    </TableCell>
                                    <TableCell className="text-right align-top pt-3">
                                        <LengthCell value={value!} />
                                    </TableCell>
                                </TableRow>
                            ))}
                        </TableBody>
                    </Table>
                </CardContent>
            </Card>
        </TooltipProvider>
    );
}
