"use client";

import { Plus, Trash2 } from "lucide-react";
import { Button } from "@/src/components/ui/button";
import { Input } from "@/src/components/ui/input";
import { Label } from "@/src/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/src/components/ui/select";
import type { TemplateSection } from "@/src/bindings";
import { useExtractorTags } from "@/src/hooks/use-extractor-tags";
import {
  CONDITION_TYPES,
  conditionInputValue,
  conditionSecondInputValue,
  buildCondition,
} from "./template-options";
import { TemplateSectionEditor } from "./TemplateSectionEditor";

export function TemplateConditionalEditor({
  section,
  onChange,
}: {
  section: Extract<TemplateSection, { kind: "conditional" }>;
  onChange: (s: TemplateSection) => void;
}) {
  const { extractorTags } = useExtractorTags();
  const when = section.when as Record<string, unknown>;
  const currentOp = (when.op as string) ?? "sitemapMissing";
  const condDef = CONDITION_TYPES.find((c) => c.value === currentOp);
  const hasSecond =
    "hasSecondInput" in (condDef ?? {}) &&
    (condDef as { hasSecondInput?: boolean }).hasSecondInput;

  const isTagCondition =
    currentOp === "tagPresent" ||
    currentOp === "tagMissing" ||
    currentOp === "tagContains";

  return (
    <div className="space-y-3">
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
            {isTagCondition ? (
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
                          <span className="text-xs text-muted-foreground">
                            {t.label}
                          </span>
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
            <Label className="text-[10px]">
              {"secondLabel" in (condDef ?? {})
                ? (condDef as { secondLabel: string }).secondLabel
                : "Value"}
            </Label>
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
              <TemplateSectionEditor
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
