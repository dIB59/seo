"use client";

interface Tag {
  name: string;
  label: string;
}

interface TagSelectorProps {
  tags: Tag[];
  selectedTags: string[];
  onSelectionChange: (tags: string[]) => void;
}

export function TagSelector({ tags, selectedTags, onSelectionChange }: TagSelectorProps) {
  if (tags.length === 0) return null;

  return (
    <div className="rounded-md border p-3 space-y-2">
      <div className="text-sm font-medium">Extractor Tags in Report</div>
      <p className="text-xs text-muted-foreground">
        Select which custom extractor tags to include in AI prompts via{" "}
        <code className="text-[10px]">{"{tag_summary}"}</code>. Unselected tags
        are excluded from the report. None selected = all included.
      </p>
      <div className="flex flex-wrap gap-2 pt-1">
        {tags.map((tag) => {
          const bare = tag.name.startsWith("tag:") ? tag.name.slice(4) : tag.name;
          const isSelected = selectedTags.includes(bare);
          return (
            <button
              key={tag.name}
              type="button"
              onClick={() => {
                onSelectionChange(
                  isSelected
                    ? selectedTags.filter((t) => t !== bare)
                    : [...selectedTags, bare],
                );
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
  );
}
