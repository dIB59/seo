import { Card, CardContent } from "@/src/components/ui/card";
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
import { FileText, Hash } from "lucide-react";
import CharLengthBadge from "@/src/app/analysis/details/_components/atoms/CharLengthBadge";
import type { PageAnalysisData } from "@/src/api/analysis";

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

  return (
    <TooltipProvider>
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
    </TooltipProvider>
  );
}
