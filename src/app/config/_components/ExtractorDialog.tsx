"use client";

import { useState } from "react";
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
  extractor?: any;
  onCreate: (data: any) => Promise<void>;
  onUpdate: (data: any) => Promise<void>;
  onCancel: () => void;
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
  const [name, setName] = useState(extractor?.name || "");
  const [displayName, setDisplayName] = useState(extractor?.display_name || "");
  const [description, setDescription] = useState(extractor?.description || "");
  const [extractorType, setExtractorType] = useState(extractor?.extractor_type || "css");
  const [selector, setSelector] = useState(extractor?.selector || "");
  const [attribute, setAttribute] = useState(extractor?.attribute || "");
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
          selector,
          attribute: attribute || null,
        });
      } else {
        await onCreate({
          name: name.toLowerCase().replace(/\s+/g, "_"),
          display_name: displayName,
          description: description || null,
          extractor_type: extractorType,
          selector,
          attribute: attribute || null,
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
              onChange={(e: any) => setName(e.target.value)}
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
          onChange={(e: any) => setDisplayName(e.target.value)}
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
          onChange={(e: any) => setDescription(e.target.value)}
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
          onChange={(e: any) => setSelector(e.target.value)}
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
            onChange={(e: any) => setAttribute(e.target.value)}
            placeholder="href"
          />
          <p className="text-xs text-muted-foreground">
            Leave empty to extract element text content
          </p>
        </div>
      )}

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
