import { Card, CardContent, CardHeader, CardTitle } from "@/src/components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/src/components/ui/table";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/src/components/ui/tooltip";
import { FileText, Hash, KeyRound } from "lucide-react";
import CharLengthBadge from "@/src/app/analysis/details/_components/atoms/CharLengthBadge";
import type { PageAnalysisData } from "@/src/api/analysis";

function formatFieldLabel(key: string): string {
  return key
    .replace(/^[^.]+\./, "")
    .replace(/[_.:-]+/g, " ")
    .replace(/\b\w/g, (match) => match.toUpperCase());
}

function toDisplayValue(value: unknown): string {
  if (value === null || value === undefined) {
    return "";
  }

  if (typeof value === "string") {
    return value;
  }

  if (typeof value === "number" || typeof value === "boolean") {
    return String(value);
  }

  try {
    return JSON.stringify(value);
  } catch {
    return String(value);
  }
}

function MetaContentValue({ value }: { value: string }) {
  const isUrl = value.startsWith("http://") || value.startsWith("https://");

  if (isUrl) {
    return (
      <Tooltip>
        <TooltipTrigger asChild>
          <a
            href={value}
            target="_blank"
            rel="noopener noreferrer"
            className="text-primary hover:underline text-sm block break-all whitespace-normal max-w-full"
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
        <span className="text-sm block break-all whitespace-normal max-w-full cursor-default">
          {value}
        </span>
      </TooltipTrigger>
      <TooltipContent className="max-w-md">
        <p className="break-words">{value}</p>
      </TooltipContent>
    </Tooltip>
  );
}

export default function MetaTab({ page }: { page: PageAnalysisData }) {
  const metaFields = [
    { label: "Title", value: page.title, maxLength: 60, icon: FileText },
    { label: "Meta Description", value: page.meta_description, maxLength: 160, icon: FileText },
    { label: "Meta Keywords", value: page.meta_keywords, icon: Hash },
    { label: "Canonical URL", value: page.canonical_url, icon: FileText },
  ];

  const extractedData = page.extracted_data || {};
  const extractedFields = Object.entries(extractedData)
    .map(([key, value]) => ({
      key,
      label: formatFieldLabel(key),
      value: toDisplayValue(value),
    }))
    .filter((field) => field.value.trim().length > 0)
    .sort((a, b) => a.label.localeCompare(b.label));

  const hasExtractedData = extractedFields.length > 0;

  return (
    <TooltipProvider>
      <div className="space-y-6">
        <Card>
          <CardContent className="pt-6">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-[150px]">Field</TableHead>
                  <TableHead>Content</TableHead>
                  <TableHead className="w-[100px] text-right">Length</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {metaFields.map(({ label, value, maxLength, icon: Icon }) => (
                  <TableRow key={label}>
                    <TableCell className="font-medium">
                      <div className="flex items-center gap-2">
                        <Icon className="h-4 w-4 text-muted-foreground" />
                        {label}
                      </div>
                    </TableCell>
                    <TableCell className="align-top whitespace-normal">
                      {value ? (
                        <MetaContentValue value={value} />
                      ) : (
                        <span className="text-muted-foreground italic">Not set</span>
                      )}
                    </TableCell>
                    <TableCell className="text-right">
                      {value ? (
                        <CharLengthBadge length={value.length} maxRecommended={maxLength} />
                      ) : (
                        <span className="text-muted-foreground">-</span>
                      )}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </CardContent>
        </Card>

        {hasExtractedData && (
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="flex items-center gap-2 text-base">
                <KeyRound className="h-4 w-4" />
                Extracted Metadata
              </CardTitle>
            </CardHeader>
            <CardContent className="pt-0">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead className="w-[220px]">Field</TableHead>
                    <TableHead>Content</TableHead>
                    <TableHead className="w-[100px] text-right">Length</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {extractedFields.map(({ key, label, value }) => (
                    <TableRow key={key}>
                      <TableCell className="font-medium">
                        <div className="flex items-center gap-2">
                          <KeyRound className="h-4 w-4 text-muted-foreground" />
                          {label}
                        </div>
                      </TableCell>
                      <TableCell className="align-top whitespace-normal">
                        {value ? (
                          <MetaContentValue value={value} />
                        ) : (
                          <span className="text-muted-foreground italic">Not set</span>
                        )}
                      </TableCell>
                      <TableCell className="text-right">
                        {value ? (
                          <CharLengthBadge length={value.length} />
                        ) : (
                          <span className="text-muted-foreground">-</span>
                        )}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </CardContent>
          </Card>
        )}
      </div>
    </TooltipProvider>
  );
}
