import useSWR from "swr";
import { listTags, type Tag } from "@/src/api/extension";

/**
 * Fetches all tags and splits them into extractor vs builtin groups.
 * Replaces the duplicated `useSWR("tags-all", ...)` + filter pattern
 * in ReportTemplateEditor, TagsSettings, and ConditionalEditor.
 */
export function useExtractorTags() {
  const { data: tags = [], isLoading } = useSWR<Tag[]>(
    "tags-all",
    () => listTags(),
  );

  const extractorTags = tags.filter((t) => t.source.kind === "extractor");
  const builtinTags = tags.filter((t) => t.source.kind === "builtin");

  return { tags, extractorTags, builtinTags, isLoading };
}
