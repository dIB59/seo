"use client";

import { useState } from "react";
import useSWR from "swr";
import { toast } from "sonner";
import { Save, Plus, Trash2, GripVertical, Type, MessageSquare, Brain, GitBranch, Minus, Heading } from "lucide-react";

import { Button } from "@/src/components/ui/button";
import { Input } from "@/src/components/ui/input";
import { Label } from "@/src/components/ui/label";
import { Textarea } from "@/src/components/ui/textarea";
import { Badge } from "@/src/components/ui/badge";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/src/components/ui/select";
import { Card, CardContent, CardHeader, CardTitle } from "@/src/components/ui/card";

import {
  listReportTemplates,
  updateReportTemplate,
  type ReportTemplate,
} from "@/src/api/report";
import { listTags, type Tag } from "@/src/api/extension";
import type { TemplateSection } from "@/src/bindings";

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
        perPatternTemplate: "**{pattern.name}** — {pattern.pct}% of pages. {pattern.recommendation}",
        emptyTemplate: "No patterns detected.",
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
  const [saving, setSaving] = useState(false);
  const [addType, setAddType] = useState<string>("text");

  // Fetch extractor tags for the tag picker
  const { data: allTags = [] } = useSWR("tags-all", () => listTags());
  const extractorTags = allTags.filter((t) => t.source.kind === "extractor");

  // Sync when active template changes
  const activeId = active?.id;
  useState(() => {
    if (active) setSections(active.sections);
  });

  async function handleSave() {
    if (!active) return;
    setSaving(true);
    try {
      const updated: ReportTemplate = { ...active, sections, selectedTags };
      await updateReportTemplate(updated);
      await mutate();
      setDirty(false);
      toast.success("Template saved");
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Failed to save template");
    } finally {
      setSaving(false);
    }
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
        <Button onClick={handleSave} disabled={!dirty || saving} size="sm">
          <Save className="h-3.5 w-3.5 mr-1.5" />
          {saving ? "Saving…" : "Save Template"}
        </Button>
      </div>

      {/* Tag selection */}
      {extractorTags.length > 0 && (
        <div className="rounded-md border p-3 space-y-2">
          <div className="text-sm font-medium">Extractor Tags in Report</div>
          <p className="text-xs text-muted-foreground">
            Select which custom extractor tags to include in AI prompts via{" "}
            <code className="text-[10px]">{"{tag_summary}"}</code>. Unselected tags
            are excluded from the report. None selected = all included.
          </p>
          <div className="flex flex-wrap gap-2 pt-1">
            {extractorTags.map((tag) => {
              // Extract the bare tag name from "tag:og_image" → "og_image"
              const bare = tag.name.startsWith("tag:") ? tag.name.slice(4) : tag.name;
              const isSelected = selectedTags.includes(bare);
              return (
                <button
                  key={tag.name}
                  type="button"
                  onClick={() => {
                    setSelectedTags((prev) =>
                      isSelected ? prev.filter((t) => t !== bare) : [...prev, bare],
                    );
                    setDirty(true);
                  }}
                  className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium border transition-colors ${
                    isSelected
                      ? "bg-primary text-primary-foreground border-primary"
                      : "bg-muted/50 text-muted-foreground border-border hover:border-primary/50"
                  }`}
                >
                  <code className="text-[10px]">{tag.name}</code>
                  <span className="opacity-70">{tag.label}</span>
                </button>
              );
            })}
          </div>
        </div>
      )}

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
              <SectionEditor
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

// ── Per-section inline editor ────────────────────────────────────────────────

function SectionEditor({
  section,
  onChange,
}: {
  section: TemplateSection;
  onChange: (s: TemplateSection) => void;
}) {
  switch (section.kind) {
    case "heading":
      return (
        <div className="grid grid-cols-[80px_1fr] gap-2">
          <div>
            <Label className="text-[10px]">Level</Label>
            <Select
              value={String(section.level)}
              onValueChange={(v) => onChange({ ...section, level: Number(v) })}
            >
              <SelectTrigger className="h-8">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {[1, 2, 3, 4, 5, 6].map((n) => (
                  <SelectItem key={n} value={String(n)}>
                    H{n}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div>
            <Label className="text-[10px]">Text</Label>
            <Input
              className="h-8 text-sm"
              value={section.text}
              onChange={(e) => onChange({ ...section, text: e.target.value })}
            />
          </div>
        </div>
      );

    case "text":
      return (
        <div>
          <Label className="text-[10px]">Template text (supports {"{variables}"})</Label>
          <Textarea
            className="text-sm min-h-[60px]"
            value={section.template}
            onChange={(e) => onChange({ ...section, template: e.target.value })}
          />
        </div>
      );

    case "ai":
      return (
        <div className="space-y-2">
          <div>
            <Label className="text-[10px]">Label (shown in progress)</Label>
            <Input
              className="h-8 text-sm"
              value={section.label}
              onChange={(e) => onChange({ ...section, label: e.target.value })}
            />
          </div>
          <div>
            <Label className="text-[10px]">Prompt (persona is prepended automatically)</Label>
            <Textarea
              className="text-sm min-h-[100px] font-mono text-xs"
              value={section.prompt}
              onChange={(e) => onChange({ ...section, prompt: e.target.value })}
            />
          </div>
        </div>
      );

    case "patternSummary":
      return (
        <div className="space-y-2">
          <div>
            <Label className="text-[10px]">Per-pattern template</Label>
            <Textarea
              className="text-sm min-h-[40px]"
              value={section.perPatternTemplate}
              onChange={(e) =>
                onChange({ ...section, perPatternTemplate: e.target.value })
              }
            />
          </div>
          <div>
            <Label className="text-[10px]">Empty template (when no patterns match)</Label>
            <Input
              className="h-8 text-sm"
              value={section.emptyTemplate ?? ""}
              onChange={(e) =>
                onChange({
                  ...section,
                  emptyTemplate: e.target.value || null,
                })
              }
            />
          </div>
        </div>
      );

    case "conditional":
      return (
        <ConditionalEditor
          section={section}
          onChange={onChange}
        />
      );

    case "divider":
      return (
        <div className="text-xs text-muted-foreground">
          Renders as a horizontal rule (---)
        </div>
      );

    default:
      return null;
  }
}

// ── Condition types for the picker ───────────────────────────────────────────

const CONDITION_TYPES = [
  { value: "sitemapMissing", label: "Sitemap is missing", hasInput: false },
  { value: "robotsMissing", label: "Robots.txt is missing", hasInput: false },
  { value: "scoreLt", label: "SEO score less than…", hasInput: true, inputLabel: "Score threshold" },
  { value: "criticalIssuesGt", label: "Critical issues greater than…", hasInput: true, inputLabel: "Count threshold" },
  { value: "patternFired", label: "Specific pattern fired…", hasInput: true, inputLabel: "Pattern ID" },
  { value: "anyPatternMatches", label: "Any pattern matches filter", hasInput: false },
  { value: "tagPresent", label: "Tag has data…", hasInput: true, inputLabel: "Tag name (e.g. og_image)" },
  { value: "tagMissing", label: "Tag is missing…", hasInput: true, inputLabel: "Tag name (e.g. og_image)" },
  { value: "tagContains", label: "Tag value contains…", hasInput: true, inputLabel: "Tag name", hasSecondInput: true, secondLabel: "Contains text" },
] as const;

function conditionOp(when: TemplateSection extends { kind: "conditional"; when: infer W } ? W : never): string {
  if (typeof when === "object" && when !== null && "op" in when) {
    return (when as { op: string }).op;
  }
  return "sitemapMissing";
}

function conditionInputValue(when: Record<string, unknown>): string {
  const op = when.op as string;
  if (op === "scoreLt" || op === "criticalIssuesGt") return String(when.value ?? "");
  if (op === "patternFired") return String(when.patternId ?? "");
  if (op === "tagPresent" || op === "tagMissing") return String(when.tag ?? "");
  if (op === "tagContains") return String(when.tag ?? "");
  return "";
}

function conditionSecondInputValue(when: Record<string, unknown>): string {
  if ((when.op as string) === "tagContains") return String(when.value ?? "");
  return "";
}

function buildCondition(op: string, inputValue: string, secondValue?: string): Record<string, unknown> {
  switch (op) {
    case "sitemapMissing":
    case "robotsMissing":
      return { op };
    case "scoreLt":
      return { op, value: Number(inputValue) || 0 };
    case "criticalIssuesGt":
      return { op, value: Number(inputValue) || 0 };
    case "patternFired":
      return { op, patternId: inputValue };
    case "anyPatternMatches":
      return { op, filter: { kind: "all" } };
    case "tagPresent":
      return { op, tag: inputValue };
    case "tagMissing":
      return { op, tag: inputValue };
    case "tagContains":
      return { op, tag: inputValue, value: secondValue ?? "" };
    default:
      return { op };
  }
}

function ConditionalEditor({
  section,
  onChange,
}: {
  section: Extract<TemplateSection, { kind: "conditional" }>;
  onChange: (s: TemplateSection) => void;
}) {
  // Fetch extractor tags for tag-aware condition dropdowns
  const { data: allTags = [] } = useSWR("tags-all", () => listTags());
  const extractorTags = allTags.filter((t) => t.source.kind === "extractor");
  const when = section.when as Record<string, unknown>;
  const currentOp = (when.op as string) ?? "sitemapMissing";
  const condDef = CONDITION_TYPES.find((c) => c.value === currentOp);
  const hasSecond = "hasSecondInput" in (condDef ?? {}) && (condDef as { hasSecondInput?: boolean }).hasSecondInput;

  return (
    <div className="space-y-3">
      {/* Condition picker */}
      <div className={`grid gap-2 ${hasSecond ? "grid-cols-3" : "grid-cols-2"}`}>
        <div>
          <Label className="text-[10px]">Condition</Label>
          <Select
            value={currentOp}
            onValueChange={(v) => {
              const newWhen = buildCondition(v, "");
              onChange({ ...section, when: newWhen as typeof section.when });
            }}
          >
            <SelectTrigger className="h-8">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {CONDITION_TYPES.map((c) => (
                <SelectItem key={c.value} value={c.value}>
                  {c.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        {condDef?.hasInput && (
          <div>
            <Label className="text-[10px]">{condDef.inputLabel}</Label>
            {/* Tag conditions use a dropdown from the registry; others use free text */}
            {currentOp === "tagPresent" || currentOp === "tagMissing" || currentOp === "tagContains" ? (
              <Select
                value={conditionInputValue(when)}
                onValueChange={(v) => {
                  const second = conditionSecondInputValue(when);
                  const newWhen = buildCondition(currentOp, v, second);
                  onChange({ ...section, when: newWhen as typeof section.when });
                }}
              >
                <SelectTrigger className="h-8">
                  <SelectValue placeholder="Select a tag…" />
                </SelectTrigger>
                <SelectContent>
                  {extractorTags.map((t) => {
                    const bare = t.name.startsWith("tag:") ? t.name.slice(4) : t.name;
                    return (
                      <SelectItem key={bare} value={bare}>
                        <span className="flex items-center gap-2">
                          <code className="text-xs">{bare}</code>
                          <span className="text-xs text-muted-foreground">{t.label}</span>
                        </span>
                      </SelectItem>
                    );
                  })}
                </SelectContent>
              </Select>
            ) : (
              <Input
                className="h-8 text-sm"
                value={conditionInputValue(when)}
                onChange={(e) => {
                  const second = conditionSecondInputValue(when);
                  const newWhen = buildCondition(currentOp, e.target.value, second);
                  onChange({ ...section, when: newWhen as typeof section.when });
                }}
              />
            )}
          </div>
        )}

        {hasSecond && (
          <div>
            <Label className="text-[10px]">{"secondLabel" in (condDef ?? {}) ? (condDef as { secondLabel: string }).secondLabel : "Value"}</Label>
            <Input
              className="h-8 text-sm"
              value={conditionSecondInputValue(when)}
              onChange={(e) => {
                const first = conditionInputValue(when);
                const newWhen = buildCondition(currentOp, first, e.target.value);
                onChange({ ...section, when: newWhen as typeof section.when });
              }}
            />
          </div>
        )}
      </div>

      {/* Children sections */}
      <div className="pl-4 border-l-2 border-primary/20 space-y-2">
        <Label className="text-[10px] text-muted-foreground">
          Sections shown when condition is true:
        </Label>
        {section.children.map((child, i) => (
          <div key={i} className="flex items-start gap-2">
            <div className="flex-1">
              <SectionEditor
                section={child}
                onChange={(updated) => {
                  const newChildren = [...section.children];
                  newChildren[i] = updated;
                  onChange({ ...section, children: newChildren });
                }}
              />
            </div>
            <Button
              variant="ghost"
              size="icon"
              className="h-6 w-6 text-destructive shrink-0"
              onClick={() => {
                const newChildren = section.children.filter((_, idx) => idx !== i);
                onChange({ ...section, children: newChildren });
              }}
            >
              <Trash2 className="h-3 w-3" />
            </Button>
          </div>
        ))}
        <Button
          variant="ghost"
          size="sm"
          className="text-xs"
          onClick={() => {
            const newChild: TemplateSection = { kind: "text", template: "" };
            onChange({ ...section, children: [...section.children, newChild] });
          }}
        >
          <Plus className="h-3 w-3 mr-1" />
          Add child section
        </Button>
      </div>
    </div>
  );
}
