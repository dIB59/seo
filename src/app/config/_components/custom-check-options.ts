export type CheckSeverity = "info" | "warning" | "critical";
export type CheckOperator = "missing" | "lt" | "gt" | "contains" | "not_contains";

export const SEVERITY_OPTIONS: { value: CheckSeverity; label: string }[] = [
  { value: "info", label: "Info" },
  { value: "warning", label: "Warning" },
  { value: "critical", label: "Critical" },
];

export const OPERATOR_OPTIONS: { value: CheckOperator; label: string }[] = [
  { value: "missing", label: "is missing" },
  { value: "lt", label: "less than" },
  { value: "gt", label: "greater than" },
  { value: "contains", label: "contains" },
  { value: "not_contains", label: "does not contain" },
];
