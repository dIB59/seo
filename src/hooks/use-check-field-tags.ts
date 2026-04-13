import useSWR from "swr";
import { listTags, type Tag } from "@/src/api/extension";

/**
 * Fetches tags scoped to the CheckField context — used by the custom
 * check and report pattern field dropdowns. Centralizes the
 * `useSWR("tags-checkField", ...)` call that was duplicated across
 * CustomCheckDialog and ReportPatternDialog.
 */
export function useCheckFieldTags() {
  const { data: tags = [], isLoading } = useSWR<Tag[]>(
    "tags-checkField",
    () => listTags("checkField"),
  );
  return { tags, isLoading };
}
