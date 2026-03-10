"use client";

import { useState } from "react";
import type { ChangeEvent } from "react";
import type {
  CreateExtractorRequest,
  ExtractorConfigInfo,
  UpdateExtractorRequest,
} from "@/src/api/extensions";
import type { RuleSeverity } from "@/src/lib/types/extension";
import { Button } from "@/src/components/ui/button";
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
import { Badge } from "@/src/components/ui/badge";

interface ExtractorDialogContentProps {
  extractor?: ExtractorConfigInfo | null;
  onCreate: (data: CreateExtractorRequest) => Promise<void>;
  onUpdate: (data: UpdateExtractorRequest) => Promise<void>;
  onCancel: () => void;
}

interface ExtractorMeta {
  category_id?: string;
  category_label?: string;
  default_rule_severity?: RuleSeverity;
  default_rule_recommendation?: string;
  default_rule_threshold_min?: number;
  default_rule_threshold_max?: number;
}

const EXTRACTOR_TYPES = [
  { value: "css", label: "CSS Selector" },
  { value: "xpath", label: "XPath" },
  { value: "json", label: "JSON Path" },
];

export function ExtractorDialogContent({
  extractor,
  onCreate,
  onUpdate,
  onCancel,
}: ExtractorDialogContentProps) {
  const extractorMeta: ExtractorMeta = (() => {
    try {
      return extractor?.post_process ? JSON.parse(extractor.post_process) : {};
    } catch {
      return {};
    }
  })();

  const [name, setName] = useState(extractor?.name || "");
  const [displayName, setDisplayName] = useState(extractor?.display_name || "");
  const [description, setDescription] = useState(extractor?.description || "");
  const [extractorType, setExtractorType] = useState(
    extractor?.extractor_type === "css_selector" ? "css" : extractor?.extractor_type || "css",
  );
  const [selector, setSelector] = useState(extractor?.selector || "");
  const [attribute, setAttribute] = useState(extractor?.attribute || "");
  const [categoryId, setCategoryId] = useState(extractorMeta?.category_id || "");
  const [categoryLabel, setCategoryLabel] = useState(extractorMeta?.category_label || "");
  const [defaultRuleSeverity, setDefaultRuleSeverity] = useState<RuleSeverity | "none">(
    extractorMeta?.default_rule_severity || "none",
  );
  const [defaultRuleRecommendation, setDefaultRuleRecommendation] = useState(
    extractorMeta?.default_rule_recommendation || "",
  );
  const [defaultRuleThresholdMin, setDefaultRuleThresholdMin] = useState(
    extractorMeta?.default_rule_threshold_min?.toString() || "",
  );
  const [defaultRuleThresholdMax, setDefaultRuleThresholdMax] = useState(
    extractorMeta?.default_rule_threshold_max?.toString() || "",
  );
  const [isSubmitting, setIsSubmitting] = useState(false);

  const isEditing = !!extractor;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsSubmitting(true);

    try {
      if (isEditing) {
        await onUpdate({
          id: extractor.id,
          name: extractor.name,
          display_name: displayName,
          description: description || null,
          extractor_type: extractorType,
          selector,
          attribute: attribute || null,
          category_id: categoryId || null,
          category_label: categoryLabel || null,
          default_rule_severity: defaultRuleSeverity === "none" ? null : defaultRuleSeverity,
          default_rule_recommendation: defaultRuleRecommendation || null,
          default_rule_threshold_min: defaultRuleThresholdMin
            ? parseFloat(defaultRuleThresholdMin)
            : null,
          default_rule_threshold_max: defaultRuleThresholdMax
            ? parseFloat(defaultRuleThresholdMax)
            : null,
        });
      } else {
        await onCreate({
          name: name.toLowerCase().replace(/\s+/g, "_"),
          display_name: displayName,
          description: description || null,
          extractor_type: extractorType,
          selector,
          attribute: attribute || null,
          category_id: categoryId || null,
          category_label: categoryLabel || null,
          default_rule_severity: defaultRuleSeverity === "none" ? null : defaultRuleSeverity,
          default_rule_recommendation: defaultRuleRecommendation || null,
          default_rule_threshold_min: defaultRuleThresholdMin
            ? parseFloat(defaultRuleThresholdMin)
            : null,
          default_rule_threshold_max: defaultRuleThresholdMax
            ? parseFloat(defaultRuleThresholdMax)
            : null,
        });
      }
      onCancel();
    } catch (error) {
      console.error("Failed to save extractor:", error);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      {!isEditing && (
        <>
          <div className="space-y-2">
            <Label htmlFor="name">Internal Name</Label>
            <Input
              id="name"
              value={name}
              onChange={(e: ChangeEvent<HTMLInputElement>) => setName(e.target.value)}
              placeholder="my_custom_extractor"
              required
            />
            <p className="text-xs text-muted-foreground">
              Unique identifier used internally (no spaces)
            </p>
          </div>
          <div className="space-y-2">
            <Label htmlFor="extractorType">Extractor Type</Label>
            <Select value={extractorType} onValueChange={setExtractorType} required>
              <SelectTrigger>
                <SelectValue placeholder="Select extractor type" />
              </SelectTrigger>
              <SelectContent>
                {EXTRACTOR_TYPES.map((type) => (
                  <SelectItem key={type.value} value={type.value}>
                    {type.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </>
      )}

      <div className="space-y-2">
        <Label htmlFor="displayName">Display Name</Label>
        <Input
          id="displayName"
          value={displayName}
          onChange={(e: ChangeEvent<HTMLInputElement>) => setDisplayName(e.target.value)}
          placeholder="Link Extractor"
          required
        />
        <p className="text-xs text-muted-foreground">User-friendly name shown in the UI</p>
      </div>

      <div className="space-y-2">
        <Label htmlFor="description">Description (optional)</Label>
        <Textarea
          id="description"
          value={description}
          onChange={(e: ChangeEvent<HTMLTextAreaElement>) => setDescription(e.target.value)}
          placeholder="Extracts all links from the page..."
          rows={2}
        />
      </div>

      <div className="space-y-2">
        <Label htmlFor="selector">
          Selector{" "}
          <Badge variant="outline" className="ml-2 text-xs">
            {extractorType === "css" && "CSS"}
            {extractorType === "xpath" && "XPath"}
            {extractorType === "json" && "JSON"}
          </Badge>
        </Label>
        <Input
          id="selector"
          value={selector}
          onChange={(e: ChangeEvent<HTMLInputElement>) => setSelector(e.target.value)}
          placeholder={
            extractorType === "css"
              ? "a[href]"
              : extractorType === "xpath"
                ? "//a/@href"
                : "$.links[*]"
          }
          required
        />
        <p className="text-xs text-muted-foreground">
          {extractorType === "css" && "CSS selector to match elements"}
          {extractorType === "xpath" && "XPath expression to select nodes"}
          {extractorType === "json" && "JSONPath to extract data"}
        </p>
      </div>

      {extractorType === "css" && (
        <div className="space-y-2">
          <Label htmlFor="attribute">HTML Attribute (optional)</Label>
          <Input
            id="attribute"
            value={attribute}
            onChange={(e: ChangeEvent<HTMLInputElement>) => setAttribute(e.target.value)}
            placeholder="href"
          />
          <p className="text-xs text-muted-foreground">
            Leave empty to extract element text content
          </p>
        </div>
      )}

      <div className="space-y-2">
        <Label htmlFor="categoryId">Category ID (optional)</Label>
        <Input
          id="categoryId"
          value={categoryId}
          onChange={(e: ChangeEvent<HTMLInputElement>) => setCategoryId(e.target.value)}
          placeholder="open_graph"
        />
        <p className="text-xs text-muted-foreground">
          Used to group extracted fields and target rules with{" "}
          <span className="font-mono">category:&lt;id&gt;</span>
        </p>
      </div>

      <div className="space-y-2">
        <Label htmlFor="categoryLabel">Category Label (optional)</Label>
        <Input
          id="categoryLabel"
          value={categoryLabel}
          onChange={(e: ChangeEvent<HTMLInputElement>) => setCategoryLabel(e.target.value)}
          placeholder="Open Graph"
        />
      </div>

      <div className="space-y-2">
        <Label htmlFor="defaultRuleSeverity">Default Rule Severity (optional)</Label>
        <Select
          value={defaultRuleSeverity}
          onValueChange={(value) => setDefaultRuleSeverity(value as RuleSeverity | "none")}
        >
          <SelectTrigger>
            <SelectValue placeholder="No default severity" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="none">No default</SelectItem>
            <SelectItem value="critical">Critical</SelectItem>
            <SelectItem value="warning">Warning</SelectItem>
            <SelectItem value="info">Info</SelectItem>
          </SelectContent>
        </Select>
      </div>

      <div className="space-y-2">
        <Label htmlFor="defaultRuleRecommendation">Default Rule Recommendation (optional)</Label>
        <Textarea
          id="defaultRuleRecommendation"
          value={defaultRuleRecommendation}
          onChange={(e: ChangeEvent<HTMLTextAreaElement>) =>
            setDefaultRuleRecommendation(e.target.value)
          }
          placeholder="Guidance pre-filled when generating templates for this extractor"
          rows={2}
        />
      </div>

      <div className="grid grid-cols-2 gap-2">
        <div className="space-y-2">
          <Label htmlFor="defaultRuleThresholdMin">Default Rule Min (optional)</Label>
          <Input
            id="defaultRuleThresholdMin"
            type="number"
            value={defaultRuleThresholdMin}
            onChange={(e: ChangeEvent<HTMLInputElement>) =>
              setDefaultRuleThresholdMin(e.target.value)
            }
            placeholder="e.g. 1"
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="defaultRuleThresholdMax">Default Rule Max (optional)</Label>
          <Input
            id="defaultRuleThresholdMax"
            type="number"
            value={defaultRuleThresholdMax}
            onChange={(e: ChangeEvent<HTMLInputElement>) =>
              setDefaultRuleThresholdMax(e.target.value)
            }
            placeholder="e.g. 160"
          />
        </div>
      </div>

      <div className="flex justify-end gap-2 pt-4">
        <Button type="button" variant="outline" onClick={onCancel}>
          Cancel
        </Button>
        <Button type="submit" disabled={isSubmitting}>
          {isSubmitting ? "Saving..." : isEditing ? "Update" : "Create"}
        </Button>
      </div>
    </form>
  );
}
