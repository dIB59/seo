/**
 * Extension Management API
 *
 * Frontend API functions for managing SEO extensions including
 * issue rules, data extractors, and audit checks.
 * Uses specta-generated bindings from @/src/bindings
 */

import { commands } from "@/src/bindings";
import type {
  ExtensionSummary as BindingsExtensionSummary,
  IssueRuleInfo as BindingsIssueRuleInfo,
  CreateRuleRequest as BindingsCreateRuleRequest,
  UpdateRuleRequest as BindingsUpdateRuleRequest,
  DataExtractorInfo as BindingsDataExtractorInfo,
  AuditCheckInfo as BindingsAuditCheckInfo,
  Result,
} from "@/src/bindings";

// ============================================================================
// Type Definitions (frontend-compatible with undefined for optional fields)
// ============================================================================

export type RuleType = "presence" | "threshold" | "regex" | "custom";
export type RuleSeverity = "critical" | "warning" | "info";
export type ExtensionCategory =
  | "seo"
  | "accessibility"
  | "performance"
  | "security"
  | "content"
  | "technical"
  | "ux"
  | "mobile";

export const EXTENSION_CATEGORIES = [
  "seo",
  "accessibility",
  "performance",
  "security",
  "content",
  "technical",
  "ux",
  "mobile",
] as const;

/**
 * CreateRuleRequest with undefined for optional fields (frontend compatibility)
 */
export interface CreateRuleRequest {
  name: string;
  category: string;
  severity: RuleSeverity;
  rule_type: RuleType;
  target_field: string;
  threshold_min?: number;
  threshold_max?: number;
  regex_pattern?: string;
  recommendation?: string;
}

/**
 * UpdateRuleRequest with undefined for optional fields (frontend compatibility)
 */
export interface UpdateRuleRequest {
  id: string;
  name?: string;
  severity?: RuleSeverity;
  threshold_min?: number;
  threshold_max?: number;
  regex_pattern?: string;
  recommendation?: string;
  is_enabled?: boolean;
}

// Re-export bindings types for read-only use
export type ExtensionSummary = BindingsExtensionSummary;
export type IssueRuleInfo = BindingsIssueRuleInfo;
export type DataExtractorInfo = BindingsDataExtractorInfo;
export type AuditCheckInfo = BindingsAuditCheckInfo;

// ============================================================================
// Internal Helper Functions
// ============================================================================

/**
 * Helper to convert specta Result to object with helper methods
 */
function toResult<T>(result: Result<T, string>): {
  isOk(): boolean;
  isErr(): boolean;
  unwrap(): T;
  unwrapErr(): string;
} {
  if (result.status === "ok") {
    return {
      isOk: () => true,
      isErr: () => false,
      unwrap: () => result.data,
      unwrapErr: () => {
        throw new Error("Called unwrapErr on Ok result");
      },
    };
  }
  return {
    isOk: () => false,
    isErr: () => true,
    unwrap: () => {
      throw new Error(`Called unwrap on Err result: ${result.error}`);
    },
    unwrapErr: () => result.error,
  };
}

/**
 * Convert frontend CreateRuleRequest to bindings CreateRuleRequest
 */
function toBindingsCreateRuleRequest(req: CreateRuleRequest): BindingsCreateRuleRequest {
  return {
    name: req.name,
    category: req.category,
    severity: req.severity,
    rule_type: req.rule_type,
    target_field: req.target_field,
    threshold_min: req.threshold_min ?? null,
    threshold_max: req.threshold_max ?? null,
    regex_pattern: req.regex_pattern ?? null,
    recommendation: req.recommendation ?? null,
  };
}

/**
 * Convert frontend UpdateRuleRequest to bindings UpdateRuleRequest
 */
function toBindingsUpdateRuleRequest(req: UpdateRuleRequest): BindingsUpdateRuleRequest {
  return {
    id: req.id,
    name: req.name ?? null,
    severity: req.severity ?? null,
    threshold_min: req.threshold_min ?? null,
    threshold_max: req.threshold_max ?? null,
    regex_pattern: req.regex_pattern ?? null,
    recommendation: req.recommendation ?? null,
    is_enabled: req.is_enabled ?? null,
  };
}

// ============================================================================
// Extension Summary
// ============================================================================

/**
 * Get a summary of the extension system
 */
export async function getExtensionSummary(): Promise<{
  isOk(): boolean;
  isErr(): boolean;
  unwrap(): ExtensionSummary;
  unwrapErr(): string;
}> {
  const result = await commands.getExtensionSummary();
  return toResult(result);
}

/**
 * Reload extensions from the database
 */
export async function reloadExtensions(): Promise<{
  isOk(): boolean;
  isErr(): boolean;
  unwrap(): ExtensionSummary;
  unwrapErr(): string;
}> {
  const result = await commands.reloadExtensions();
  return toResult(result);
}

// ============================================================================
// Issue Rules
// ============================================================================

/**
 * Get all registered issue rules
 */
export async function getAllIssueRules(): Promise<{
  isOk(): boolean;
  isErr(): boolean;
  unwrap(): IssueRuleInfo[];
  unwrapErr(): string;
}> {
  const result = await commands.getAllIssueRules();
  return toResult(result);
}

/**
 * Create a new custom issue rule
 */
export async function createCustomRule(request: CreateRuleRequest): Promise<{
  isOk(): boolean;
  isErr(): boolean;
  unwrap(): IssueRuleInfo;
  unwrapErr(): string;
}> {
  const bindingsRequest = toBindingsCreateRuleRequest(request);
  const result = await commands.createCustomRule(bindingsRequest);
  return toResult(result);
}

/**
 * Update an existing custom rule
 */
export async function updateCustomRule(request: UpdateRuleRequest): Promise<{
  isOk(): boolean;
  isErr(): boolean;
  unwrap(): IssueRuleInfo;
  unwrapErr(): string;
}> {
  const bindingsRequest = toBindingsUpdateRuleRequest(request);
  const result = await commands.updateCustomRule(bindingsRequest);
  return toResult(result);
}

/**
 * Delete a custom rule
 */
export async function deleteCustomRule(ruleId: string): Promise<{
  isOk(): boolean;
  isErr(): boolean;
  unwrap(): void;
  unwrapErr(): string;
}> {
  const result = await commands.deleteCustomRule(ruleId);
  return toResult(result);
}

/**
 * Toggle a rule's enabled status
 */
export async function toggleRuleEnabled(
  ruleId: string,
  enabled: boolean,
): Promise<{
  isOk(): boolean;
  isErr(): boolean;
  unwrap(): IssueRuleInfo;
  unwrapErr(): string;
}> {
  const result = await commands.toggleRuleEnabled(ruleId, enabled);
  return toResult(result);
}

// ============================================================================
// Data Extractors
// ============================================================================

/**
 * Get all registered data extractors
 */
export async function getAllExtractors(): Promise<{
  isOk(): boolean;
  isErr(): boolean;
  unwrap(): DataExtractorInfo[];
  unwrapErr(): string;
}> {
  const result = await commands.getAllExtractors();
  return toResult(result);
}

// ============================================================================
// Audit Checks
// ============================================================================

/**
 * Get all registered audit checks
 */
export async function getAllAuditChecks(): Promise<{
  isOk(): boolean;
  isErr(): boolean;
  unwrap(): AuditCheckInfo[];
  unwrapErr(): string;
}> {
  const result = await commands.getAllAuditChecks();
  return toResult(result);
}

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Filter rules by various criteria
 */
export function filterRules(
  rules: IssueRuleInfo[],
  filter: {
    category?: string;
    severity?: string;
    is_builtin?: boolean;
    is_enabled?: boolean;
    search?: string;
  },
): IssueRuleInfo[] {
  return rules.filter((rule) => {
    if (filter.category && rule.category !== filter.category) return false;
    if (filter.severity && rule.severity !== filter.severity) return false;
    if (filter.is_builtin !== undefined && rule.is_builtin !== filter.is_builtin) return false;
    if (filter.is_enabled !== undefined && rule.is_enabled !== filter.is_enabled) return false;
    if (filter.search) {
      const searchLower = filter.search.toLowerCase();
      return (
        rule.name.toLowerCase().includes(searchLower) ||
        rule.id.toLowerCase().includes(searchLower) ||
        (rule.recommendation?.toLowerCase().includes(searchLower) ?? false)
      );
    }
    return true;
  });
}

/**
 * Sort rules by various criteria
 */
export function sortRules(
  rules: IssueRuleInfo[],
  sortBy: "name" | "category" | "severity" | "type",
  ascending: boolean = true,
): IssueRuleInfo[] {
  const sorted = [...rules].sort((a, b) => {
    let comparison = 0;
    switch (sortBy) {
      case "name":
        comparison = a.name.localeCompare(b.name);
        break;
      case "category":
        comparison = a.category.localeCompare(b.category);
        break;
      case "severity":
        const severityOrder: Record<string, number> = { critical: 0, warning: 1, info: 2 };
        comparison = severityOrder[a.severity] - severityOrder[b.severity];
        break;
      case "type":
        comparison = a.rule_type.localeCompare(b.rule_type);
        break;
    }
    return ascending ? comparison : -comparison;
  });
  return sorted;
}

/**
 * Group rules by category
 */
export function groupRulesByCategory(rules: IssueRuleInfo[]): Map<string, IssueRuleInfo[]> {
  const groups = new Map<string, IssueRuleInfo[]>();
  for (const rule of rules) {
    const category = rule.category || "other";
    const existing = groups.get(category) || [];
    existing.push(rule);
    groups.set(category, existing);
  }
  return groups;
}

/**
 * Group rules by severity
 */
export function groupRulesBySeverity(rules: IssueRuleInfo[]): Map<string, IssueRuleInfo[]> {
  const groups = new Map<string, IssueRuleInfo[]>();
  for (const rule of rules) {
    const existing = groups.get(rule.severity) || [];
    existing.push(rule);
    groups.set(rule.severity, existing);
  }
  return groups;
}
