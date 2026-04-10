"use client";

import { Input } from "@/src/components/ui/input";
import { Label } from "@/src/components/ui/label";
import { Textarea } from "@/src/components/ui/textarea";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/src/components/ui/select";
import type { TemplateSection } from "@/src/bindings";
import { TemplateConditionalEditor } from "./TemplateConditionalEditor";

export function TemplateSectionEditor({
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
        <TemplateConditionalEditor section={section} onChange={onChange} />
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
