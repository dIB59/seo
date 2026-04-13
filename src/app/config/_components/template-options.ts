/** Condition types for the template conditional editor picker. */
export const CONDITION_TYPES = [
  { value: "sitemapMissing", label: "Sitemap is missing", hasInput: false },
  { value: "robotsMissing", label: "Robots.txt is missing", hasInput: false },
  { value: "scoreLt", label: "SEO score less than…", hasInput: true, inputLabel: "Score threshold" },
  { value: "criticalIssuesGt", label: "Critical issues greater than…", hasInput: true, inputLabel: "Count threshold" },
  { value: "patternFired", label: "Specific pattern fired…", hasInput: true, inputLabel: "Pattern ID" },
  { value: "anyPatternMatches", label: "Any pattern matches filter", hasInput: false },
  { value: "tagPresent", label: "Tag has data…", hasInput: true, inputLabel: "Tag name" },
  { value: "tagMissing", label: "Tag is missing…", hasInput: true, inputLabel: "Tag name" },
  { value: "tagContains", label: "Tag value contains…", hasInput: true, inputLabel: "Tag name", hasSecondInput: true, secondLabel: "Contains text" },
] as const;

export function conditionInputValue(when: Record<string, unknown>): string {
  const op = when.op as string;
  if (op === "scoreLt" || op === "criticalIssuesGt") return String(when.value ?? "");
  if (op === "patternFired") return String(when.patternId ?? "");
  if (op === "tagPresent" || op === "tagMissing") return String(when.tag ?? "");
  if (op === "tagContains") return String(when.tag ?? "");
  return "";
}

export function conditionSecondInputValue(when: Record<string, unknown>): string {
  if ((when.op as string) === "tagContains") return String(when.value ?? "");
  return "";
}

export function buildCondition(op: string, inputValue: string, secondValue?: string): Record<string, unknown> {
  switch (op) {
    case "sitemapMissing":
    case "robotsMissing":
      return { op };
    case "scoreLt":
      return { op, value: Number(inputValue) || 0 };
    case "criticalIssuesGt":
      return { op, value: Number(inputValue) || 0 };
    case "patternFired":
      return { op, patternId: inputValue };
    case "anyPatternMatches":
      return { op, filter: { kind: "all" } };
    case "tagPresent":
      return { op, tag: inputValue };
    case "tagMissing":
      return { op, tag: inputValue };
    case "tagContains":
      return { op, tag: inputValue, value: secondValue ?? "" };
    default:
      return { op };
  }
}
