"use no memo";

import { useRef } from "react";
import type { SeoIssue } from "@/src/api/analysis";
import {
  Accordion,
  AccordionItem,
  AccordionTrigger,
  AccordionContent,
} from "@radix-ui/react-accordion";
import { CheckCircle2, ExternalLink } from "lucide-react";
import { Button } from "@/src/components/ui/button";
import { open } from "@tauri-apps/plugin-shell";
import { Card, CardContent } from "@/src/components/ui/card";
import { IssueIcon } from "../atoms/IssueIcon";
import { useVirtualizer } from "@tanstack/react-virtual";

function VirtualIssuePageList({ pages }: { pages: SeoIssue[] }) {
  const parentRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: pages.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 40, // Approximate height of each row
    overscan: 5,
  });

  return (
    <div
      ref={parentRef}
      className="h-[200px] overflow-y-auto pr-2 scrollbar-thin scrollbar-thumb-border scrollbar-track-transparent"
    >
      <div
        style={{
          height: `${virtualizer.getTotalSize()}px`,
          width: "100%",
          position: "relative",
        }}
      >
        {virtualizer.getVirtualItems().map((virtualItem) => {
          const issue = pages[virtualItem.index];
          return (
            <div
              key={virtualItem.key}
              style={{
                position: "absolute",
                top: 0,
                left: 0,
                width: "100%",
                height: `${virtualItem.size}px`,
                transform: `translateY(${virtualItem.start}px)`,
              }}
              className="py-1" // Add padding to separate items slightly if needed
            >
              <div className="flex items-center justify-between p-2 rounded-md bg-muted/20 border border-transparent hover:border-border/40 transition-colors h-full">
                <p className="text-xs font-mono text-muted-foreground truncate flex-1">
                  {issue.page_url}
                </p>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-6 w-6 opacity-0 group-hover:opacity-100 shrink-0"
                  onClick={(e) => {
                    e.stopPropagation();
                    open(issue.page_url);
                  }}
                >
                  <ExternalLink className="h-3 w-3" />
                </Button>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

function IssueRoot({ children, value }: { children: React.ReactNode; value: string }) {
  return (
    <AccordionItem
      value={value}
      className="group border border-white/5 rounded-xl bg-card/30 backdrop-blur-sm overflow-hidden transition-all duration-200 hover:border-white/10 hover:shadow-md data-[state=open]:bg-card/50 data-[state=open]:shadow-md data-[state=open]:border-white/10"
    >
      {children}
    </AccordionItem>
  );
}

function IssueTrigger({ children }: { children: React.ReactNode }) {
  return (
    <AccordionTrigger className="w-full flex hover:no-underline px-4 py-3 transition-colors hover:bg-primary/5 active:bg-primary/10 data-[state=open]:bg-primary/5 group-hover:bg-white/[0.02]">
      {children}
    </AccordionTrigger>
  );
}

function IssueHeader({
  iconType,
  title,
  count,
  badgeLabel,
}: {
  iconType: string;
  title: string;
  count: number;
  badgeLabel: string;
}) {
  return (
    <div className="flex items-center gap-4 w-full text-left pr-4">
      <div className="shrink-0 p-2 rounded-lg bg-background/50 border border-border/50">
        <IssueIcon type={iconType} />
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2 mb-1">
          <span className="font-medium truncate">{title}</span>
        </div>
        <span className="text-xs text-muted-foreground font-mono">
          {count} {count === 1 ? "page" : "pages"} affected
        </span>
      </div>
      <div className="hidden sm:flex items-center gap-2 px-3 py-1.5 rounded-md bg-muted/30 border border-transparent group-hover:border-primary/10 group-hover:bg-primary/5 text-xs font-medium text-muted-foreground group-hover:text-primary transition-all">
        {badgeLabel}
      </div>
    </div>
  );
}

function IssueContent({ children }: { children: React.ReactNode }) {
  return <AccordionContent className="px-4 pb-4">{children}</AccordionContent>;
}

function DescriptionBlock({ text }: { text?: string }) {
  return (
    <div className="space-y-1.5">
      <p className="text-xs uppercase tracking-wider text-muted-foreground font-medium">
        Description
      </p>
      <p className="text-sm text-foreground/80 leading-relaxed">{text}</p>
    </div>
  );
}

function RecommendationBlock({ text }: { text?: string }) {
  return (
    <div className="space-y-1.5">
      <p className="text-xs uppercase tracking-wider text-muted-foreground font-medium">
        Recommendation
      </p>
      <div className="p-3 bg-primary/5 border border-primary/10 rounded-lg">
        <p className="text-sm text-primary/90 leading-relaxed">{text}</p>
      </div>
    </div>
  );
}

export function IssuesAccordion({ issues }: { issues: SeoIssue[] }) {
  const groupedIssues: Record<string, SeoIssue[]> = {};
  issues.forEach((issue) => {
    if (!groupedIssues[issue.title]) groupedIssues[issue.title] = [];
    groupedIssues[issue.title].push(issue);
  });

  if (Object.keys(groupedIssues).length === 0) {
    return (
      <Card>
        <CardContent className="p-6 text-center">
          <CheckCircle2 className="h-12 w-12 text-success mx-auto mb-2" />
          <p className="text-muted-foreground">No issues found. Great job!</p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Accordion type="multiple" className="space-y-3">
      {Object.entries(groupedIssues).map(([title, issueGroup]) => (
        <IssueRoot key={title} value={title}>
          <IssueTrigger>
            <IssueHeader
              iconType={issueGroup[0].severity}
              title={title}
              count={issueGroup.length}
              badgeLabel="View Details"
            />
          </IssueTrigger>

          <IssueContent>
            <div className="space-y-4 pt-2 mt-2">
              <div className="grid md:grid-cols-2 gap-4">
                <DescriptionBlock text={issueGroup[0].description} />
                <RecommendationBlock text={issueGroup[0].recommendation} />
              </div>

              <div className="space-y-2">
                <p className="text-xs uppercase tracking-wider text-muted-foreground font-medium">
                  Affected URLs
                </p>
                <VirtualIssuePageList pages={issueGroup} />
              </div>
            </div>
          </IssueContent>
        </IssueRoot>
      ))}
    </Accordion>
  );
}
