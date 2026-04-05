"use client";

import { useState } from "react";
import useSWR from "swr";
import { Plus, Trash2, Pencil, X, Check, ChevronDown, ChevronUp, Sparkles } from "lucide-react";
import { toast } from "sonner";

import { Button } from "@/src/components/ui/button";
import { Input } from "@/src/components/ui/input";
import { Label } from "@/src/components/ui/label";
import { Switch } from "@/src/components/ui/switch";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/src/components/ui/dialog";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/src/components/ui/table";
import { Badge } from "@/src/components/ui/badge";
import { Separator } from "@/src/components/ui/separator";

import {
  listCustomExtractors,
  createCustomExtractor,
  updateCustomExtractor,
  deleteCustomExtractor,
  type CustomExtractor,
  type CustomExtractorParams,
} from "@/src/api/extension";
import { SelectorLivePreview } from "./SelectorLivePreview";

// ---------------------------------------------------------------------------
// Presets
// ---------------------------------------------------------------------------

interface Preset {
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

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------

const EMPTY_PARAMS: CustomExtractorParams = {
  name: "",
  key: "",
  selector: "",
  attribute: null,
  multiple: false,
  enabled: true,
};

export function ExtractorsSettings() {
  const { data: extractors = [], mutate } = useSWR("custom-extractors", listCustomExtractors);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editing, setEditing] = useState<CustomExtractor | null>(null);
  const [form, setForm] = useState<CustomExtractorParams>(EMPTY_PARAMS);
  const [saving, setSaving] = useState(false);
  const [presetsOpen, setPresetsOpen] = useState(false);

  function openCreate() {
    setEditing(null);
    setForm(EMPTY_PARAMS);
    setPresetsOpen(false);
    setDialogOpen(true);
  }

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

  function openEdit(extractor: CustomExtractor) {
    setEditing(extractor);
    setForm({
      name: extractor.name,
      key: extractor.key,
      selector: extractor.selector,
      attribute: extractor.attribute,
      multiple: extractor.multiple,
      enabled: extractor.enabled,
    });
    setPresetsOpen(false);
    setDialogOpen(true);
  }

  async function handleSave() {
    if (!form.name.trim() || !form.key.trim() || !form.selector.trim()) {
      toast.error("Name, key, and selector are required");
      return;
    }
    setSaving(true);
    try {
      if (editing) {
        const updated = await updateCustomExtractor(editing.id, form);
        mutate((prev = []) => prev.map((e) => (e.id === editing.id ? updated : e)), false);
        toast.success("Extractor updated");
      } else {
        const created = await createCustomExtractor(form);
        mutate((prev = []) => [...prev, created], false);
        toast.success("Extractor created");
      }
      setDialogOpen(false);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Failed to save extractor");
    } finally {
      setSaving(false);
    }
  }

  async function handleDelete(id: string) {
    try {
      await deleteCustomExtractor(id);
      mutate((prev = []) => prev.filter((e) => e.id !== id), false);
      toast.success("Extractor deleted");
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Failed to delete extractor");
    }
  }

  async function handleToggleEnabled(extractor: CustomExtractor) {
    try {
      const updated = await updateCustomExtractor(extractor.id, {
        ...extractor,
        attribute: extractor.attribute ?? null,
        enabled: !extractor.enabled,
      });
      mutate((prev = []) => prev.map((e) => (e.id === extractor.id ? updated : e)), false);
    } catch {
      toast.error("Failed to toggle extractor");
    }
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <p className="text-sm text-muted-foreground">
          Extractors pull custom data from every crawled page using CSS selectors.
        </p>
        <Button size="sm" onClick={openCreate}>
          <Plus className="h-4 w-4 mr-1" />
          Add Extractor
        </Button>
      </div>

      {extractors.length === 0 ? (
        <p className="text-sm text-muted-foreground py-8 text-center">
          No extractors configured.
        </p>
      ) : (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Name</TableHead>
              <TableHead>Key</TableHead>
              <TableHead>Selector</TableHead>
              <TableHead>Mode</TableHead>
              <TableHead>Enabled</TableHead>
              <TableHead className="w-20" />
            </TableRow>
          </TableHeader>
          <TableBody>
            {extractors.map((e) => (
              <TableRow key={e.id}>
                <TableCell className="font-medium">{e.name}</TableCell>
                <TableCell>
                  <code className="text-xs bg-muted px-1 py-0.5 rounded">{e.key}</code>
                </TableCell>
                <TableCell>
                  <code className="text-xs bg-muted px-1 py-0.5 rounded">{e.selector}</code>
                  {e.attribute && (
                    <Badge variant="outline" className="ml-1 text-xs">
                      @{e.attribute}
                    </Badge>
                  )}
                </TableCell>
                <TableCell>
                  <Badge variant={e.multiple ? "secondary" : "outline"} className="text-xs">
                    {e.multiple ? "all" : "first"}
                  </Badge>
                </TableCell>
                <TableCell>
                  <Switch checked={e.enabled} onCheckedChange={() => handleToggleEnabled(e)} />
                </TableCell>
                <TableCell>
                  <div className="flex gap-1">
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-7 w-7"
                      onClick={() => openEdit(e)}
                    >
                      <Pencil className="h-3.5 w-3.5" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-7 w-7 text-destructive hover:text-destructive"
                      onClick={() => handleDelete(e.id)}
                    >
                      <Trash2 className="h-3.5 w-3.5" />
                    </Button>
                  </div>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      )}

      {/* ------------------------------------------------------------------ */}
      {/* Dialog                                                             */}
      {/* ------------------------------------------------------------------ */}
      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        {/* Wide enough for the two-column live preview */}
        <DialogContent className="sm:max-w-4xl max-h-[92vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>{editing ? "Edit Extractor" : "New Extractor"}</DialogTitle>
          </DialogHeader>

          {/* Presets panel — only shown when creating */}
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

          {/* Form fields */}
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

            {/* Live preview — always visible, reacts to selector + attribute + HTML edits */}
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
            <Button variant="ghost" onClick={() => setDialogOpen(false)}>
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
    </div>
  );
}

// ---------------------------------------------------------------------------
// PresetRow
// ---------------------------------------------------------------------------

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
