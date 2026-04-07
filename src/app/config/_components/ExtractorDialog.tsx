"use client";

import { useEffect, useState } from "react";
import { Check, ChevronDown, ChevronUp, Sparkles, X } from "lucide-react";

import { Button } from "@/src/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/src/components/ui/dialog";
import { Input } from "@/src/components/ui/input";
import { Label } from "@/src/components/ui/label";
import { Separator } from "@/src/components/ui/separator";
import { Switch } from "@/src/components/ui/switch";

import type { CustomExtractor, CustomExtractorParams } from "@/src/api/extension";
import { SelectorLivePreview } from "./SelectorLivePreview";

export interface Preset {
  name: string;
  key: string;
  selector: string;
  attribute: string | null;
  multiple: boolean;
  description: string;
  htmlPreview: string;
  highlightValue: string;
}

const PRESETS: Preset[] = [
  {
    name: "Hreflang Tags",
    key: "hreflang",
    selector: "link[rel='alternate'][hreflang]",
    attribute: "hreflang",
    multiple: true,
    description: "Collects all language/region codes declared on the page (e.g. en-US, fr-FR).",
    htmlPreview: `<link rel="alternate" hreflang="en-US" href="..." />`,
    highlightValue: "en-US",
  },
  {
    name: "OG Image",
    key: "og_image",
    selector: "meta[property='og:image']",
    attribute: "content",
    multiple: false,
    description: "The Open Graph image URL used when sharing on social media.",
    htmlPreview: `<meta property="og:image" content="https://example.com/img.jpg" />`,
    highlightValue: "https://example.com/img.jpg",
  },
  {
    name: "OG Title",
    key: "og_title",
    selector: "meta[property='og:title']",
    attribute: "content",
    multiple: false,
    description: "The title shown when the page is shared on social media.",
    htmlPreview: `<meta property="og:title" content="My Page Title" />`,
    highlightValue: "My Page Title",
  },
  {
    name: "Canonical URL",
    key: "canonical",
    selector: "link[rel='canonical']",
    attribute: "href",
    multiple: false,
    description: "The preferred URL for this page, used to avoid duplicate content issues.",
    htmlPreview: `<link rel="canonical" href="https://example.com/page" />`,
    highlightValue: "https://example.com/page",
  },
  {
    name: "JSON-LD Schema",
    key: "schema_types",
    selector: "script[type='application/ld+json']",
    attribute: null,
    multiple: true,
    description: "Extracts structured data blocks for schema.org markup.",
    htmlPreview: `<script type="application/ld+json">{"@type": "Article"}</script>`,
    highlightValue: `{"@type": "Article"}`,
  },
  {
    name: "Robots Meta",
    key: "robots_meta",
    selector: "meta[name='robots']",
    attribute: "content",
    multiple: false,
    description: "The robots directive (e.g. noindex, nofollow) on this page.",
    htmlPreview: `<meta name="robots" content="noindex, follow" />`,
    highlightValue: "noindex, follow",
  },
  {
    name: "Author",
    key: "author",
    selector: "meta[name='author']",
    attribute: "content",
    multiple: false,
    description: "The author name as declared in the page head.",
    htmlPreview: `<meta name="author" content="Jane Smith" />`,
    highlightValue: "Jane Smith",
  },
  {
    name: "H1 Heading",
    key: "h1_text",
    selector: "h1",
    attribute: null,
    multiple: false,
    description: "The text content of the main H1 heading.",
    htmlPreview: `<h1>Welcome to Our Store</h1>`,
    highlightValue: "Welcome to Our Store",
  },
];

const EMPTY_PARAMS: CustomExtractorParams = {
  name: "",
  key: "",
  selector: "",
  attribute: null,
  multiple: false,
  enabled: true,
};

function paramsFrom(extractor: CustomExtractor): CustomExtractorParams {
  return {
    name: extractor.name,
    key: extractor.key,
    selector: extractor.selector,
    attribute: extractor.attribute,
    multiple: extractor.multiple,
    enabled: extractor.enabled,
  };
}

interface ExtractorDialogProps {
  open: boolean;
  editing: CustomExtractor | null;
  saving: boolean;
  onOpenChange: (open: boolean) => void;
  onSave: (params: CustomExtractorParams) => void;
  onValidationError: (message: string) => void;
}

export function ExtractorDialog({
  open,
  editing,
  saving,
  onOpenChange,
  onSave,
  onValidationError,
}: ExtractorDialogProps) {
  const [form, setForm] = useState<CustomExtractorParams>(EMPTY_PARAMS);
  const [presetsOpen, setPresetsOpen] = useState(false);

  useEffect(() => {
    if (!open) return;
    setForm(editing ? paramsFrom(editing) : EMPTY_PARAMS);
    setPresetsOpen(false);
  }, [open, editing]);

  function applyPreset(preset: Preset) {
    setForm({
      name: preset.name,
      key: preset.key,
      selector: preset.selector,
      attribute: preset.attribute,
      multiple: preset.multiple,
      enabled: true,
    });
    setPresetsOpen(false);
  }

  function handleSave() {
    if (!form.name.trim() || !form.key.trim() || !form.selector.trim()) {
      onValidationError("Name, key, and selector are required");
      return;
    }
    onSave(form);
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-4xl max-h-[92vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{editing ? "Edit Extractor" : "New Extractor"}</DialogTitle>
        </DialogHeader>

        {!editing && (
          <div className="rounded-lg border border-border/60 bg-muted/30 overflow-hidden">
            <button
              type="button"
              className="w-full flex items-center justify-between px-4 py-3 text-sm font-medium hover:bg-muted/50 transition-colors"
              onClick={() => setPresetsOpen((o) => !o)}
            >
              <span className="flex items-center gap-2">
                <Sparkles className="h-4 w-4 text-primary" />
                Common extractor examples — click one to fill in the form
              </span>
              {presetsOpen ? (
                <ChevronUp className="h-4 w-4 text-muted-foreground" />
              ) : (
                <ChevronDown className="h-4 w-4 text-muted-foreground" />
              )}
            </button>

            {presetsOpen && (
              <div className="border-t border-border/60 divide-y divide-border/40">
                {PRESETS.map((preset) => (
                  <PresetRow key={preset.key} preset={preset} onApply={applyPreset} />
                ))}
              </div>
            )}
          </div>
        )}

        <div className="space-y-4 py-2">
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-1.5">
              <Label htmlFor="ext-name">Name</Label>
              <Input
                id="ext-name"
                placeholder="OG Image"
                value={form.name}
                onChange={(e) => setForm((f) => ({ ...f, name: e.target.value }))}
              />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="ext-key">
                Key{" "}
                <span className="text-xs text-muted-foreground font-normal">
                  — used in reports &amp; checks
                </span>
              </Label>
              <Input
                id="ext-key"
                placeholder="og_image"
                value={form.key}
                onChange={(e) => setForm((f) => ({ ...f, key: e.target.value }))}
              />
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-1.5">
              <Label htmlFor="ext-selector">CSS Selector</Label>
              <Input
                id="ext-selector"
                placeholder="meta[property='og:image']"
                value={form.selector}
                onChange={(e) => setForm((f) => ({ ...f, selector: e.target.value }))}
              />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="ext-attr">
                Attribute{" "}
                <span className="text-xs text-muted-foreground font-normal">— optional</span>
              </Label>
              <Input
                id="ext-attr"
                placeholder='e.g. "content" or "href" — blank = element text'
                value={form.attribute ?? ""}
                onChange={(e) =>
                  setForm((f) => ({ ...f, attribute: e.target.value || null }))
                }
              />
            </div>
          </div>

          <SelectorLivePreview selector={form.selector} attribute={form.attribute} />

          <Separator />

          <div className="flex items-center gap-6 flex-wrap">
            <label className="flex items-center gap-2 text-sm cursor-pointer">
              <Switch
                checked={form.multiple}
                onCheckedChange={(v) => setForm((f) => ({ ...f, multiple: v }))}
              />
              <span>
                Collect <strong>all</strong> matches{" "}
                <span className="text-muted-foreground font-normal">
                  (off = first match only)
                </span>
              </span>
            </label>
            <label className="flex items-center gap-2 text-sm cursor-pointer">
              <Switch
                checked={form.enabled}
                onCheckedChange={(v) => setForm((f) => ({ ...f, enabled: v }))}
              />
              Enabled
            </label>
          </div>
        </div>

        <DialogFooter>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            <X className="h-4 w-4 mr-1" />
            Cancel
          </Button>
          <Button onClick={handleSave} disabled={saving}>
            <Check className="h-4 w-4 mr-1" />
            {editing ? "Save Changes" : "Create"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function PresetRow({ preset, onApply }: { preset: Preset; onApply: (p: Preset) => void }) {
  const parts = preset.htmlPreview.split(preset.highlightValue);

  return (
    <button
      type="button"
      onClick={() => onApply(preset)}
      className="w-full text-left px-4 py-3 hover:bg-muted/60 transition-colors group"
    >
      <div className="flex items-start justify-between gap-4">
        <div className="min-w-0 space-y-1">
          <p className="text-sm font-medium group-hover:text-primary transition-colors">
            {preset.name}
          </p>
          <p className="text-xs text-muted-foreground">{preset.description}</p>
          <p className="font-mono text-xs text-muted-foreground/70 mt-1 break-all">
            {parts[0]}
            <span className="bg-primary/20 text-foreground rounded px-0.5 font-semibold">
              {preset.highlightValue}
            </span>
            {parts[1]}
          </p>
        </div>
        <span className="shrink-0 text-xs text-primary opacity-0 group-hover:opacity-100 transition-opacity pt-0.5">
          Use this →
        </span>
      </div>
    </button>
  );
}
