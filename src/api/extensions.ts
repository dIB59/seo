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
  IssueGeneratorInfo,
  CreateRuleRequest as BindingsCreateRuleRequest,
  UpdateRuleRequest as BindingsUpdateRuleRequest,
  DataExtractorInfo as BindingsDataExtractorInfo,
  AuditCheckInfo as BindingsAuditCheckInfo,
  ExtractorConfigInfo as BindingsExtractorConfigInfo,
  RuleFieldInfo as BindingsRuleFieldInfo,
  RuleTargetMigrationResult as BindingsRuleTargetMigrationResult,
  CreateExtractorRequest as BindingsCreateExtractorRequest,
  UpdateExtractorRequest as BindingsUpdateExtractorRequest,
  Result,
} from "@/src/bindings";

// Re-export the canonical binding shapes used by the frontend.
export type ExtensionSummary = BindingsExtensionSummary;
export type IssueRuleInfo = IssueGeneratorInfo;
export type DataExtractorInfo = BindingsDataExtractorInfo;
export type AuditCheckInfo = BindingsAuditCheckInfo;
export type CreateRuleRequest = BindingsCreateRuleRequest;
export type UpdateRuleRequest = BindingsUpdateRuleRequest;
export type CreateExtractorRequest = BindingsCreateExtractorRequest;
export type UpdateExtractorRequest = BindingsUpdateExtractorRequest;
export type ExtractorConfigInfo = BindingsExtractorConfigInfo;

const EXTENSION_CATEGORIES = [
  "seo",
  "accessibility",
  "performance",
  "security",
  "content",
  "technical",
  "ux",
  "mobile",
] as const;

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

// ============================================================================
// Extension Summary
// ============================================================================

/**
 * Get a summary of the extension system
 */
export async function getExtensionSummary(): Promise<{
  isOk(): boolean;
  isErr(): boolean;
  unwrap(): BindingsExtensionSummary;
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
  unwrap(): BindingsExtensionSummary;
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
  unwrap(): IssueGeneratorInfo[];
  unwrapErr(): string;
}> {
  const result = await commands.getAllIssueRules();
  return toResult(result);
}

/**
 * Create a new custom issue rule
 */
export async function createCustomRule(request: BindingsCreateRuleRequest): Promise<{
  isOk(): boolean;
  isErr(): boolean;
  unwrap(): IssueGeneratorInfo;
  unwrapErr(): string;
}> {
  const result = await commands.createCustomRule(request);
  return toResult(result);
}

/**
 * Update an existing custom rule
 */
export async function updateCustomRule(request: BindingsUpdateRuleRequest): Promise<{
  isOk(): boolean;
  isErr(): boolean;
  unwrap(): IssueGeneratorInfo;
  unwrapErr(): string;
}> {
  const result = await commands.updateCustomRule(request);
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
  unwrap(): IssueGeneratorInfo;
  unwrapErr(): string;
}> {
  const result = await commands.toggleRuleEnabled(ruleId, enabled);
  return toResult(result);
}

/**
 * Normalize legacy rule target syntax to field:* format
 */
export async function normalizeRuleTargetFields(): Promise<{
  isOk(): boolean;
  isErr(): boolean;
  unwrap(): BindingsRuleTargetMigrationResult;
  unwrapErr(): string;
}> {
  const result = await commands.normalizeRuleTargetFields();
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
  unwrap(): BindingsDataExtractorInfo[];
  unwrapErr(): string;
}> {
  const result = await commands.getAllExtractors();
  return toResult(result);
}

/**
 * Get all extractor configs from database (including custom ones)
 */
export async function getExtractorConfigs(): Promise<{
  isOk(): boolean;
  isErr(): boolean;
  unwrap(): BindingsExtractorConfigInfo[];
  unwrapErr(): string;
}> {
  const result = await commands.getExtractorConfigs();
  return toResult(result);
}

/**
 * Get registry entries for rule-targetable fields
 */
export async function getRuleFieldRegistry(): Promise<{
  isOk(): boolean;
  isErr(): boolean;
  unwrap(): BindingsRuleFieldInfo[];
  unwrapErr(): string;
}> {
  const result = await commands.getRuleFieldRegistry();
  return toResult(result);
}

/**
 * Create a new custom extractor
 */
export async function createCustomExtractor(request: BindingsCreateExtractorRequest): Promise<{
  isOk(): boolean;
  isErr(): boolean;
  unwrap(): BindingsExtractorConfigInfo;
  unwrapErr(): string;
}> {
  const result = await commands.createCustomExtractor(request);
  return toResult(result);
}

/**
 * Update an existing custom extractor
 */
export async function updateCustomExtractor(request: BindingsUpdateExtractorRequest): Promise<{
  isOk(): boolean;
  isErr(): boolean;
  unwrap(): BindingsExtractorConfigInfo;
  unwrapErr(): string;
}> {
  const result = await commands.updateCustomExtractor(request);
  return toResult(result);
}

/**
 * Delete a custom extractor
 */
export async function deleteCustomExtractor(extractorId: string): Promise<{
  isOk(): boolean;
  isErr(): boolean;
  unwrap(): void;
  unwrapErr(): string;
}> {
  const result = await commands.deleteCustomExtractor(extractorId);
  return toResult(result);
}

/**
 * Toggle an extractor's enabled status
 */
export async function toggleExtractorEnabled(
  extractorId: string,
  enabled: boolean,
): Promise<{
  isOk(): boolean;
  isErr(): boolean;
  unwrap(): BindingsExtractorConfigInfo;
  unwrapErr(): string;
}> {
  const result = await commands.toggleExtractorEnabled(extractorId, enabled);
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
  unwrap(): BindingsAuditCheckInfo[];
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
  rules: IssueGeneratorInfo[],
  filter: {
    category?: string;
    severity?: string;
    is_builtin?: boolean;
    is_enabled?: boolean;
    search?: string;
  },
): IssueGeneratorInfo[] {
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
  rules: IssueGeneratorInfo[],
  sortBy: "name" | "category" | "severity" | "type",
  ascending: boolean = true,
): IssueGeneratorInfo[] {
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
        {
          const severityOrder: Record<string, number> = { critical: 0, warning: 1, info: 2 };
          comparison = severityOrder[a.severity] - severityOrder[b.severity];
        }
        break;
    }
    return ascending ? comparison : -comparison;
  });
  return sorted;
}

/**
 * Group rules by category
 */
function groupRulesByCategory(rules: IssueGeneratorInfo[]): Map<string, IssueGeneratorInfo[]> {
  const groups = new Map<string, IssueGeneratorInfo[]>();
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
function groupRulesBySeverity(rules: IssueGeneratorInfo[]): Map<string, IssueGeneratorInfo[]> {
  const groups = new Map<string, IssueGeneratorInfo[]>();
  for (const rule of rules) {
    const existing = groups.get(rule.severity) || [];
    existing.push(rule);
    groups.set(rule.severity, existing);
  }
  return groups;
}
