import { useState } from "react";
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
  Link2,
  ExternalLink,
  Globe,
  FileText,
  Link as LinkIcon,
  AlertTriangle,
  type LucideIcon,
} from "lucide-react";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/src/components/ui/tooltip";
import { Badge } from "@/src/components/ui/badge";
import type { LinkDetail } from "@/src/api/analysis";
import { cn } from "@/src/lib/utils";

function LinkStatCard({
  label,
  count,
  icon: Icon,
  onClick,
  active,
  variant = "default",
}: {
  label: string;
  count: number;
  icon: LucideIcon;
  onClick: () => void;
  active: boolean;
  variant?: "default" | "destructive";
}) {
  return (
    <div
      onClick={onClick}
      className={cn(
        "cursor-pointer p-4 rounded-xl border transition-all duration-200",
        variant === "destructive" && !active && "border-destructive/20 bg-destructive/5",
        active
          ? "bg-primary/10 border-primary/30 ring-1 ring-primary/20"
          : "bg-card/50 border-border/40 hover:bg-card/80 hover:border-border/60",
      )}
    >
      <div className="flex items-center justify-between mb-2">
        <div
          className={cn(
            "p-2 rounded-lg",
            active ? "bg-primary/20 text-primary" : "bg-muted text-muted-foreground",
          )}
        >
          <Icon className="h-4 w-4" />
        </div>
        <span
          className={cn(
            "text-2xl font-bold font-mono",
            active ? "text-primary" : "text-foreground",
          )}
        >
          {count}
        </span>
      </div>
      <p className="text-xs font-medium text-muted-foreground uppercase tracking-wider">{label}</p>
    </div>
  );
}

export default function LinksTab({ links }: { links: LinkDetail[] }) {
  const [filter, setFilter] = useState<
    "all" | "internal" | "subdomain" | "external" | "resource" | "broken"
  >("all");

  if (!links || links.length === 0) {
    return (
      <Card>
        <CardContent className="py-12 text-center">
          <Link2 className="h-12 w-12 text-muted-foreground mx-auto mb-3" />
          <p className="text-muted-foreground">No links found on this page</p>
          <p className="text-sm text-muted-foreground mt-1">
            Check settings to ensure links are being crawled
          </p>
        </CardContent>
      </Card>
    );
  }

  const internalLinks = links.filter((l) => l.link_type === "internal");
  const subdomainLinks = links.filter((l) => l.link_type === "subdomain");
  const externalLinks = links.filter((l) => l.link_type === "external");
  const resourceLinks = links.filter((l) => l.link_type === "resource");
  const brokenLinks = links.filter((l) => l.status_code && l.status_code >= 400);

  const filteredLinks = links.filter((link) => {
    if (filter === "all") return true;
    if (filter === "internal") return link.link_type === "internal";
    if (filter === "subdomain") return link.link_type === "subdomain";
    if (filter === "external") return link.link_type === "external";
    if (filter === "resource") return link.link_type === "resource";
    if (filter === "broken") return link.status_code && link.status_code >= 400;
    return true;
  });

  return (
    <div className="space-y-6">
      <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-5 gap-4">
        <LinkStatCard
          label="Internal"
          count={internalLinks.length}
          icon={LinkIcon}
          onClick={() => setFilter("internal")}
          active={filter === "internal"}
        />
        <LinkStatCard
          label="Subdomains"
          count={subdomainLinks.length}
          icon={Globe}
          onClick={() => setFilter("subdomain")}
          active={filter === "subdomain"}
        />
        <LinkStatCard
          label="External"
          count={externalLinks.length}
          icon={ExternalLink}
          onClick={() => setFilter("external")}
          active={filter === "external"}
        />
        <LinkStatCard
          label="Resource"
          count={resourceLinks.length}
          icon={FileText}
          onClick={() => setFilter("resource")}
          active={filter === "resource"}
        />
        <LinkStatCard
          label="Broken"
          count={brokenLinks.length}
          icon={AlertTriangle}
          onClick={() => setFilter("broken")}
          active={filter === "broken"}
          variant={brokenLinks.length > 0 ? "destructive" : "default"}
        />
      </div>

      <Card>
        <CardContent className="pt-0">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>URL</TableHead>
                <TableHead>Anchor Text</TableHead>
                <TableHead className="w-[80px] text-center">Type</TableHead>
                <TableHead className="w-[80px] text-center">Status</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {filteredLinks.map((link, idx) => (
                <TableRow key={idx}>
                  <TableCell className="max-w-[250px]">
                    <TooltipProvider>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <span className="text-sm truncate block font-mono text-muted-foreground cursor-default">
                            {link.href}
                          </span>
                        </TooltipTrigger>
                        <TooltipContent>
                          <p className="max-w-md break-all font-mono text-xs">{link.href}</p>
                        </TooltipContent>
                      </Tooltip>
                    </TooltipProvider>
                  </TableCell>
                  <TableCell>
                    {link.text ? (
                      <TooltipProvider>
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <span className="text-sm truncate block max-w-[300px] cursor-default">
                              {link.text}
                            </span>
                          </TooltipTrigger>
                          <TooltipContent>
                            <p className="max-w-md break-words">{link.text}</p>
                          </TooltipContent>
                        </Tooltip>
                      </TooltipProvider>
                    ) : (
                      <span className="text-muted-foreground italic">No text</span>
                    )}
                  </TableCell>
                  <TableCell className="text-center">
                    <Badge
                      variant="outline"
                      className={cn(
                        "text-xs",
                        link.link_type === "external"
                          ? "bg-muted"
                          : link.link_type === "subdomain"
                            ? "bg-blue-100 text-blue-800 border-blue-200"
                            : "bg-primary/15 text-primary border-primary/20",
                      )}
                    >
                      {link.link_type === "external"
                        ? "Ext"
                        : link.link_type === "subdomain"
                          ? "Sub"
                          : link.link_type === "resource"
                            ? "Res"
                            : "Int"}
                    </Badge>
                  </TableCell>
                  <TableCell className="text-center">
                    {link.status_code ? (
                      <Badge
                        variant="outline"
                        className={cn(
                          "text-xs font-mono",
                          link.status_code >= 200 && link.status_code < 300
                            ? "bg-success/15 text-success border-success/20"
                            : link.status_code >= 400
                              ? "bg-destructive/15 text-destructive border-destructive/20"
                              : "bg-warning/15 text-warning border-warning/20",
                        )}
                      >
                        {link.status_code}
                      </Badge>
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
    </div>
  );
}
