"use client";

import { useState } from "react";
import useSWR from "swr";
import { toast } from "sonner";
import { Save, Plus, Trash2, GripVertical, Type, MessageSquare, Brain, GitBranch, Minus, Heading } from "lucide-react";

import { Button } from "@/src/components/ui/button";
import { Badge } from "@/src/components/ui/badge";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/src/components/ui/select";
import { Card, CardContent, CardHeader } from "@/src/components/ui/card";

import { useExtractorTags } from "@/src/hooks/use-extractor-tags";
import { useMutation } from "@/src/hooks/use-mutation";
import {
  listReportTemplates,
  updateReportTemplate,
  type ReportTemplate,
} from "@/src/api/report";
import type { TemplateSection } from "@/src/bindings";
import { TemplateSectionEditor } from "./TemplateSectionEditor";
import { TagSelector } from "./TagSelector";

const SECTION_TYPES = [
  { value: "heading", label: "Heading", icon: Heading },
  { value: "text", label: "Text Block", icon: Type },
  { value: "ai", label: "AI Prompt", icon: Brain },
  { value: "patternSummary", label: "Pattern Summary", icon: MessageSquare },
  { value: "conditional", label: "Conditional", icon: GitBranch },
  { value: "divider", label: "Divider", icon: Minus },
] as const;

function sectionIcon(section: TemplateSection) {
  const kind = section.kind;
  const entry = SECTION_TYPES.find((t) => t.value === kind);
  if (!entry) return <Type className="h-4 w-4" />;
  const Icon = entry.icon;
  return <Icon className="h-4 w-4" />;
}

function sectionLabel(section: TemplateSection): string {
  switch (section.kind) {
    case "heading":
      return `H${section.level}: ${section.text}`;
    case "text":
      return section.template.slice(0, 60) + (section.template.length > 60 ? "…" : "");
    case "ai":
      return `AI: ${section.label}`;
    case "patternSummary":
      return `Patterns: ${section.filter.kind}`;
    case "conditional":
      return `If: ${section.when.op}`;
    case "divider":
      return "---";
    default:
      return "Unknown";
  }
}

function makeSectionOfType(kind: string): TemplateSection {
  switch (kind) {
    case "heading":
      return { kind: "heading", level: 2, text: "New Section" };
    case "text":
      return { kind: "text", template: "Enter text with {url}, {score}, {tag.name} variables." };
    case "ai":
      return { kind: "ai", label: "New AI Section", prompt: "Analyze {url} and provide insights." };
    case "patternSummary":
      return {
        kind: "patternSummary",
        filter: { kind: "topN", n: 3 },
        per_pattern_template: "**{pattern.name}** — {pattern.pct}% of pages. {pattern.recommendation}",
        empty_template: "No patterns detected.",
      };
    case "conditional":
      return {
        kind: "conditional",
        when: { op: "sitemapMissing" },
        children: [{ kind: "text", template: "Sitemap is missing!" }],
      };
    case "divider":
      return { kind: "divider" };
    default:
      return { kind: "text", template: "" };
  }
}

export function ReportTemplateEditor() {
  const { data: templates = [], mutate } = useSWR("report-templates", listReportTemplates);
  const active = templates.find((t) => t.id === "default") ?? templates[0];

  const [sections, setSections] = useState<TemplateSection[]>(active?.sections ?? []);
  const [selectedTags, setSelectedTags] = useState<string[]>(active?.selectedTags ?? []);
  const [dirty, setDirty] = useState(false);
  const [addType, setAddType] = useState<string>("text");

  const { extractorTags } = useExtractorTags();

  const saveTemplate = useMutation(
    async (template: ReportTemplate) => {
      await updateReportTemplate(template);
      await mutate();
      setDirty(false);
    },
    { successMessage: "Template saved" },
  );

  function handleSave() {
    if (!active) return;
    saveTemplate.execute({ ...active, sections, selectedTags });
  }

  function addSection() {
    const newSection = makeSectionOfType(addType);
    setSections((prev) => [...prev, newSection]);
    setDirty(true);
  }

  function removeSection(index: number) {
    setSections((prev) => prev.filter((_, i) => i !== index));
    setDirty(true);
  }

  function moveSection(from: number, direction: "up" | "down") {
    const to = direction === "up" ? from - 1 : from + 1;
    if (to < 0 || to >= sections.length) return;
    setSections((prev) => {
      const next = [...prev];
      [next[from], next[to]] = [next[to], next[from]];
      return next;
    });
    setDirty(true);
  }

  function updateSection(index: number, updated: TemplateSection) {
    setSections((prev) => prev.map((s, i) => (i === index ? updated : s)));
    setDirty(true);
  }

  if (!active) {
    return <p className="text-muted-foreground">No templates found.</p>;
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div className="text-sm font-medium">
          Template: <span className="text-muted-foreground">{active.name}</span>
          {active.isBuiltin && (
            <Badge variant="outline" className="ml-2 text-[10px]">
              Built-in
            </Badge>
          )}
        </div>
        <Button onClick={handleSave} disabled={!dirty || saveTemplate.isLoading} size="sm">
          <Save className="h-3.5 w-3.5 mr-1.5" />
          {saveTemplate.isLoading ? "Saving…" : "Save Template"}
        </Button>
      </div>

      <TagSelector
        tags={extractorTags}
        selectedTags={selectedTags}
        onSelectionChange={(tags) => {
          setSelectedTags(tags);
          setDirty(true);
        }}
      />

      {/* Section list */}
      <div className="space-y-2">
        {sections.map((section, i) => (
          <Card key={i} className="group">
            <CardHeader className="p-3 pb-0 flex flex-row items-center gap-2">
              <div className="flex items-center gap-1 text-muted-foreground">
                <button
                  className="p-0.5 hover:text-foreground disabled:opacity-30"
                  disabled={i === 0}
                  onClick={() => moveSection(i, "up")}
                >
                  <GripVertical className="h-3.5 w-3.5 rotate-180" />
                </button>
                <button
                  className="p-0.5 hover:text-foreground disabled:opacity-30"
                  disabled={i === sections.length - 1}
                  onClick={() => moveSection(i, "down")}
                >
                  <GripVertical className="h-3.5 w-3.5" />
                </button>
              </div>
              <div className="flex items-center gap-2 flex-1 min-w-0">
                {sectionIcon(section)}
                <Badge variant="secondary" className="text-[10px] shrink-0">
                  {SECTION_TYPES.find((t) => t.value === section.kind)?.label ?? section.kind}
                </Badge>
                <span className="text-xs text-muted-foreground truncate">
                  {sectionLabel(section)}
                </span>
              </div>
              <Button
                variant="ghost"
                size="icon"
                className="h-6 w-6 opacity-0 group-hover:opacity-100 text-destructive"
                onClick={() => removeSection(i)}
              >
                <Trash2 className="h-3 w-3" />
              </Button>
            </CardHeader>
            <CardContent className="p-3 pt-2">
              <TemplateSectionEditor
                section={section}
                onChange={(updated) => updateSection(i, updated)}
              />
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Add section */}
      <div className="flex items-center gap-2 pt-2">
        <Select value={addType} onValueChange={setAddType}>
          <SelectTrigger className="w-[200px]">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {SECTION_TYPES.map((t) => (
              <SelectItem key={t.value} value={t.value}>
                {t.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Button variant="outline" size="sm" onClick={addSection}>
          <Plus className="h-3.5 w-3.5 mr-1" />
          Add Section
        </Button>
      </div>
    </div>
  );
}

