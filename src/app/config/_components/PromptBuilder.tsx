"use client";

import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  DragEndEvent,
} from "@dnd-kit/core";
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
  useSortable,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { Button } from "@/src/components/ui/button";
import { toast } from "sonner";
import { Textarea } from "@/src/components/ui/textarea";
import { Trash2, Plus, GripVertical, RotateCcw } from "lucide-react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/src/components/ui/dropdown-menu";
import type { PromptBlock } from "@/src/lib/types";

// Re-export for backward compatibility
export type { PromptBlock };

const VARIABLE_OPTIONS = [
  { id: "url", label: "Website URL", template: "Website URL: {url}" },
  { id: "score", label: "SEO Score", template: "SEO Score: {score}" },
  { id: "pages", label: "Pages Analyzed", template: "Pages Analyzed: {pages_count}" },
  {
    id: "issues",
    label: "Total Issues Breakdown",
    template:
      "Total Issues: {total_issues}\n- Critical: {critical_issues}\n- Warnings: {warning_issues}",
  },
  { id: "top_issues", label: "Top 5 Issues List", template: "Top Issues Found:\n{top_issues}" },
  {
    id: "metrics",
    label: "Site Metrics",
    template: "Site Metrics:\n- Load Time: {avg_load_time}s\n- Total Words: {total_words}",
  },
  { id: "ssl", label: "SSL Status", template: "SSL Certificate: {ssl_certificate}" },
  { id: "sitemap", label: "Sitemap Status", template: "Sitemap Found: {sitemap_found}" },
  { id: "robots", label: "Robots.txt Status", template: "Robots.txt Found: {robots_txt_found}" },
];

function SortableBlock({
  block,
  onRemove,
  onUpdate,
}: {
  block: PromptBlock;
  onRemove: (id: string) => void;
  onUpdate: (id: string, val: string) => void;
}) {
  const { attributes, listeners, setNodeRef, transform, transition } = useSortable({
    id: block.id,
  });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      className="flex gap-2 bg-card p-3 rounded-lg border border-border/50 hover:border-border/80 transition-colors group mb-3 relative shadow-sm"
    >
      {/* Controls */}
      <div className="flex flex-col gap-1 items-center justify-start pt-1">
        <div
          {...attributes}
          {...listeners}
          className="p-1 rounded cursor-grab active:cursor-grabbing text-muted-foreground/50 hover:bg-muted hover:text-foreground transition-colors"
        >
          <GripVertical className="h-4 w-4" />
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 space-y-2">
        <div className="flex justify-between items-center">
          <span
            className={`text-[10px] uppercase font-bold tracking-wider px-2 py-0.5 rounded-full border ${block.type === "variable" ? "text-blue-500 bg-blue-500/10 border-blue-500/20" : "text-zinc-500 bg-zinc-500/10 border-zinc-500/20"}`}
          >
            {block.type === "variable" ? "Data Variable" : "Instruction Text"}
          </span>
          <Button
            variant="ghost"
            size="icon"
            className="h-6 w-6 text-muted-foreground hover:text-destructive opacity-0 group-hover:opacity-100 transition-opacity"
            onClick={() => onRemove(block.id)}
          >
            <Trash2 className="h-3.5 w-3.5" />
          </Button>
        </div>
        <Textarea
          value={block.content}
          onChange={(e) => onUpdate(block.id, e.target.value)}
          className={`text-sm font-mono bg-background/50 border-border/50 focus:bg-background resize-y ${block.type === "text" ? "min-h-[100px]" : "min-h-[60px]"}`}
          placeholder={
            block.type === "variable"
              ? "Variable template (e.g. Score: {score})"
              : "Enter instructions for the AI..."
          }
        />
      </div>
    </div>
  );
}

interface PromptBuilderProps {
  blocks: PromptBlock[];
  setBlocks: (blocks: PromptBlock[]) => void;
}

export function PromptBuilder({ blocks, setBlocks }: PromptBuilderProps) {
  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    }),
  );

  const handleReset = () => {
    setBlocks([
      {
        id: "intro",
        type: "text",
        content: "Please provide:\n1. A brief executive summary...\n\nData to include:",
      },
      ...VARIABLE_OPTIONS.map((v) => ({
        id: v.id,
        type: "variable" as const,
        content: v.template,
      })),
    ]);
    toast.info("Reset to defaults (unsaved)");
  };

  // Helper functions
  const addTextBlock = () => {
    setBlocks([...blocks, { id: `text-${Date.now()}`, type: "text", content: "" }]);
  };
  const addVariableBlock = (template: string) => {
    setBlocks([...blocks, { id: `var-${Date.now()}`, type: "variable", content: template }]);
  };
  const removeBlock = (id: string) => setBlocks(blocks.filter((b) => b.id !== id));
  const updateBlockContent = (id: string, content: string) =>
    setBlocks(blocks.map((b) => (b.id === id ? { ...b, content } : b)));

  // Drag handling
  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    if (active.id !== over?.id) {
      const oldIndex = blocks.findIndex((item) => item.id === active.id);
      const newIndex = blocks.findIndex((item) => item.id === over?.id);

      const newBlocks = arrayMove(blocks, oldIndex, newIndex);
      setBlocks(newBlocks);
    }
  };

  return (
    <div className="space-y-6 animate-in fade-in slide-in-from-bottom-2 duration-300">
      <div className="flex items-center justify-between">
        <div className="space-y-1">
          <p className="text-sm text-muted-foreground">
            Design the prompt structure sent to Gemini. Blocks are processed top-to-bottom.
          </p>
        </div>
        <Button variant="outline" size="sm" onClick={handleReset}>
          <RotateCcw className="h-3.5 w-3.5 mr-2" />
          Reset
        </Button>
      </div>

      <div className="bg-muted/10 p-1 rounded-xl border border-border/40 min-h-[400px]">
        <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={handleDragEnd}>
          <SortableContext items={blocks.map((b) => b.id)} strategy={verticalListSortingStrategy}>
            {blocks.map((block) => (
              <SortableBlock
                key={block.id}
                block={block}
                onRemove={removeBlock}
                onUpdate={updateBlockContent}
              />
            ))}
          </SortableContext>
        </DndContext>

        {/* Add Buttons Area */}
        <div className="p-4 flex justify-center gap-3 border-t border-border/40 border-dashed mt-2">
          <Button variant="outline" size="sm" onClick={addTextBlock}>
            <Plus className="h-3.5 w-3.5 mr-2" /> Add Instruction
          </Button>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="outline" size="sm">
                <Plus className="h-3.5 w-3.5 mr-2" /> Add Data Variable
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent>
              {VARIABLE_OPTIONS.map((opt) => (
                <DropdownMenuItem key={opt.id} onClick={() => addVariableBlock(opt.template)}>
                  {opt.label}
                </DropdownMenuItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>
    </div>
  );
}
