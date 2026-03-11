"use client";

import { useState, useEffect, useCallback } from "react";
import {
  Puzzle,
  AlertTriangle,
  Database,
  CheckCircle2,
  Plus,
  Trash2,
  Edit,
  RefreshCw,
  Search,
  ToggleLeft,
  ToggleRight,
} from "lucide-react";
import { Button } from "@/src/components/ui/button";
import { Card, CardContent } from "@/src/components/ui/card";
import { Input } from "@/src/components/ui/input";
import { Badge } from "@/src/components/ui/badge";
import { Skeleton } from "@/src/components/ui/skeleton";
import { Separator } from "@/src/components/ui/separator";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/src/components/ui/select";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/src/components/ui/dialog";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/src/components/ui/alert-dialog";
import { toast } from "sonner";
import { RuleDialog as EnhancedRuleDialog } from "./RuleDialog";
import { ExtractorDialogContent } from "./ExtractorDialog";
import {
  getExtensionSummary,
  getAllIssueRules,
  getAllExtractors,
  getExtractorConfigs,
  getAllAuditChecks,
  createCustomRule,
  updateCustomRule,
  deleteCustomRule,
  toggleRuleEnabled,
  createCustomExtractor,
  updateCustomExtractor,
  deleteCustomExtractor,
  toggleExtractorEnabled,
  normalizeRuleTargetFields,
  reloadExtensions,
  filterRules,
  sortRules,
  type ExtensionSummary,
  type IssueRuleInfo,
  type DataExtractorInfo,
  type AuditCheckInfo,
  type CreateRuleRequest,
  type UpdateRuleRequest,
  type CreateExtractorRequest,
  type UpdateExtractorRequest,
  type ExtractorConfigInfo,
} from "@/src/api/extensions";
import type { RuleType } from "@/src/lib/types/extension";

interface SummaryCardsProps {
  summary: ExtensionSummary | null;
  isLoading: boolean;
}

interface ExtractorRulePresetMeta {
  default_rule_severity?: string;
  default_rule_recommendation?: string;
  default_rule_threshold_min?: number;
  default_rule_threshold_max?: number;
}

function parseExtractorRulePreset(postProcess: string | null): ExtractorRulePresetMeta {
  if (!postProcess) {
    return {};
  }

  try {
    const parsed = JSON.parse(postProcess) as ExtractorRulePresetMeta;
    return {
      default_rule_severity: parsed.default_rule_severity,
      default_rule_recommendation: parsed.default_rule_recommendation,
      default_rule_threshold_min: parsed.default_rule_threshold_min,
      default_rule_threshold_max: parsed.default_rule_threshold_max,
    };
  } catch {
    return {};
  }
}

function formatPresetThreshold(min: number | undefined, max: number | undefined): string | null {
  if (min === undefined && max === undefined) {
    return null;
  }

  if (min !== undefined && max !== undefined) {
    return `${min} - ${max}`;
  }

  if (min !== undefined) {
    return `>= ${min}`;
  }

  return `<= ${max}`;
}

function SummaryCards({ summary, isLoading }: SummaryCardsProps) {
  if (isLoading) {
    return (
      <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
        {[1, 2, 3].map((index) => (
          <Card key={index} className="bg-card/50">
            <CardContent className="p-4">
              <Skeleton className="h-16 w-full" />
            </CardContent>
          </Card>
        ))}
      </div>
    );
  }

  const cards = [
    {
      title: "Issue Rules",
      value: summary?.total_validators ?? 0,
      description: `${summary?.builtin_count ?? 0} built-in, ${summary?.custom_count ?? 0} custom`,
      icon: AlertTriangle,
      color: "text-orange-500",
    },
    {
      title: "Data Extractors",
      value: summary?.total_extractors ?? 0,
      description: "Active extractors",
      icon: Database,
      color: "text-blue-500",
    },
    {
      title: "Audit Checks",
      value: summary?.total_exporters ?? 0,
      description: "Scoring checks",
      icon: CheckCircle2,
      color: "text-green-500",
    },
  ];

  return (
    <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
      {cards.map((card) => (
        <Card key={card.title} className="bg-card/50 transition-colors hover:bg-card/70">
          <CardContent className="p-4">
            <div className="flex items-center gap-3">
              <div className={`rounded-lg bg-muted p-2 ${card.color}`}>
                <card.icon className="h-5 w-5" />
              </div>
              <div>
                <p className="text-2xl font-bold">{card.value}</p>
                <p className="text-xs text-muted-foreground">{card.title}</p>
                <p className="text-xs text-muted-foreground">{card.description}</p>
              </div>
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}

interface RuleListItemProps {
  rule: IssueRuleInfo;
  onToggle: (id: string, enabled: boolean) => void;
  onEdit: (rule: IssueRuleInfo) => void;
  onDelete: (id: string) => void;
}

function RuleListItem({ rule, onToggle, onEdit, onDelete }: RuleListItemProps) {
  const ruleType = rule.rule_type as RuleType;

  return (
    <div
      className={`flex items-center justify-between rounded-lg border p-3 transition-colors ${
        rule.is_enabled ? "bg-card/50" : "bg-muted/30 opacity-60"
      }`}
    >
      <div className="flex min-w-0 flex-1 items-center gap-3">
        <button onClick={() => onToggle(rule.id, !rule.is_enabled)} className="flex-shrink-0">
          {rule.is_enabled ? (
            <ToggleRight className="h-5 w-5 text-green-500" />
          ) : (
            <ToggleLeft className="h-5 w-5 text-muted-foreground" />
          )}
        </button>
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <p className="truncate font-medium">{rule.name}</p>
            {rule.is_builtin && (
              <Badge variant="outline" className="text-xs">
                Built-in
              </Badge>
            )}
          </div>
          <div className="mt-1 flex items-center gap-2">
            <span className="text-xs font-medium capitalize text-muted-foreground">
              {rule.severity}
            </span>
            <span className="text-xs text-muted-foreground">{rule.category}</span>
            <span className="text-xs text-muted-foreground">•</span>
            <span className="text-xs text-muted-foreground">{ruleType}</span>
          </div>
        </div>
      </div>
      <div className="flex items-center gap-1">
        {!rule.is_builtin && (
          <>
            <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => onEdit(rule)}>
              <Edit className="h-4 w-4" />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8 text-destructive hover:text-destructive"
              onClick={() => onDelete(rule.id)}
            >
              <Trash2 className="h-4 w-4" />
            </Button>
          </>
        )}
      </div>
    </div>
  );
}

export function ExtensionsSettings() {
  const [summary, setSummary] = useState<ExtensionSummary | null>(null);
  const [rules, setRules] = useState<IssueRuleInfo[]>([]);
  const [extractors, setExtractors] = useState<DataExtractorInfo[]>([]);
  const [checks, setChecks] = useState<AuditCheckInfo[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<"rules" | "extractors" | "checks">("rules");

  const [searchQuery, setSearchQuery] = useState("");
  const [categoryFilter, setCategoryFilter] = useState<string>("all");
  const [severityFilter, setSeverityFilter] = useState<string>("all");
  const [showOnlyCustom, setShowOnlyCustom] = useState(false);

  const [isRuleDialogOpen, setIsRuleDialogOpen] = useState(false);
  const [editingRule, setEditingRule] = useState<IssueRuleInfo | null>(null);
  const [deletingRuleId, setDeletingRuleId] = useState<string | null>(null);

  const [isExtractorDialogOpen, setIsExtractorDialogOpen] = useState(false);
  const [editingExtractor, setEditingExtractor] = useState<ExtractorConfigInfo | null>(null);
  const [deletingExtractorId, setDeletingExtractorId] = useState<string | null>(null);
  const [extractorConfigs, setExtractorConfigs] = useState<ExtractorConfigInfo[]>([]);

  const loadData = useCallback(async (showLoader = true) => {
    if (showLoader) {
      setIsLoading(true);
    }

    try {
      await normalizeRuleTargetFields();

      const [summaryRes, rulesRes, extractorsRes, checksRes, configsRes] = await Promise.all([
        getExtensionSummary(),
        getAllIssueRules(),
        getAllExtractors(),
        getAllAuditChecks(),
        getExtractorConfigs(),
      ]);

      if (summaryRes.isOk()) setSummary(summaryRes.unwrap());
      if (rulesRes.isOk()) setRules(rulesRes.unwrap());
      if (extractorsRes.isOk()) setExtractors(extractorsRes.unwrap());
      if (checksRes.isOk()) setChecks(checksRes.unwrap());
      if (configsRes.isOk()) setExtractorConfigs(configsRes.unwrap());
    } catch (error) {
      console.error("Failed to load extensions:", error);
      toast.error("Failed to load extension data");
    }

    if (showLoader) {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    loadData(false);
  }, [loadData]);

  const handleToggleRule = async (id: string, enabled: boolean) => {
    const result = await toggleRuleEnabled(id, enabled);
    if (result.isOk()) {
      setRules((previous) =>
        previous.map((rule) => (rule.id === id ? { ...rule, is_enabled: enabled } : rule)),
      );
      toast.success(enabled ? "Rule enabled" : "Rule disabled");
    } else {
      toast.error("Failed to toggle rule");
    }
  };

  const handleCreateRule = async (data: CreateRuleRequest) => {
    const result = await createCustomRule(data);
    if (result.isOk()) {
      setRules((previous) => [...previous, result.unwrap()]);
      toast.success("Rule created successfully");
    } else {
      const error = result.isErr() ? result.unwrapErr() : "Failed to create rule";
      toast.error(error);
      throw new Error(error);
    }
  };

  const handleUpdateRule = async (data: UpdateRuleRequest) => {
    const result = await updateCustomRule(data);
    if (result.isOk()) {
      setRules((previous) =>
        previous.map((rule) => (rule.id === data.id ? result.unwrap() : rule)),
      );
      toast.success("Rule updated successfully");
    } else {
      const error = result.isErr() ? result.unwrapErr() : "Failed to update rule";
      toast.error(error);
      throw new Error(error);
    }
  };

  const handleDeleteRule = async () => {
    if (!deletingRuleId) return;

    const result = await deleteCustomRule(deletingRuleId);
    if (result.isOk()) {
      setRules((previous) => previous.filter((rule) => rule.id !== deletingRuleId));
      toast.success("Rule deleted successfully");
    } else {
      toast.error("Failed to delete rule");
    }
    setDeletingRuleId(null);
  };

  const handleToggleExtractor = async (id: string, enabled: boolean) => {
    const result = await toggleExtractorEnabled(id, enabled);
    if (result.isOk()) {
      setExtractorConfigs((previous) =>
        previous.map((extractor) =>
          extractor.id === id ? { ...extractor, is_enabled: enabled } : extractor,
        ),
      );
      toast.success(enabled ? "Extractor enabled" : "Extractor disabled");
    } else {
      toast.error("Failed to toggle extractor");
    }
  };

  const handleCreateExtractor = async (data: CreateExtractorRequest) => {
    const result = await createCustomExtractor(data);
    if (result.isOk()) {
      setExtractorConfigs((previous) => [...previous, result.unwrap()]);
      toast.success("Extractor created successfully");
    } else {
      const error = result.isErr() ? result.unwrapErr() : "Failed to create extractor";
      toast.error(error);
      throw new Error(error);
    }
  };

  const handleUpdateExtractor = async (data: UpdateExtractorRequest) => {
    const result = await updateCustomExtractor(data);
    if (result.isOk()) {
      setExtractorConfigs((previous) =>
        previous.map((extractor) => (extractor.id === data.id ? result.unwrap() : extractor)),
      );
      toast.success("Extractor updated successfully");
    } else {
      const error = result.isErr() ? result.unwrapErr() : "Failed to update extractor";
      toast.error(error);
      throw new Error(error);
    }
  };

  const handleDeleteExtractor = async () => {
    if (!deletingExtractorId) return;

    const result = await deleteCustomExtractor(deletingExtractorId);
    if (result.isOk()) {
      setExtractorConfigs((previous) =>
        previous.filter((extractor) => extractor.id !== deletingExtractorId),
      );
      toast.success("Extractor deleted successfully");
    } else {
      toast.error("Failed to delete extractor");
    }
    setDeletingExtractorId(null);
  };

  const handleReload = async () => {
    setIsLoading(true);
    const result = await reloadExtensions();
    if (result.isOk()) {
      await loadData();
      toast.success("Extensions reloaded");
    } else {
      toast.error("Failed to reload extensions");
      setIsLoading(false);
    }
  };

  const filteredRules = filterRules(rules, {
    category: categoryFilter !== "all" ? categoryFilter : undefined,
    severity: severityFilter !== "all" ? severityFilter : undefined,
    is_builtin: showOnlyCustom ? false : undefined,
    search: searchQuery || undefined,
  });

  const sortedRules = sortRules(filteredRules, "name");
  const availableRuleCategories = [
    ...new Set(rules.map((rule) => rule.category).filter(Boolean)),
  ].sort((left, right) => left.localeCompare(right));

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="flex items-center gap-2 text-lg font-semibold">
            <Puzzle className="h-5 w-5" />
            Extension System
          </h3>
          <p className="text-sm text-muted-foreground">
            Manage issue rules, data extractors, and audit checks.
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={handleReload} disabled={isLoading}>
            <RefreshCw className={`mr-2 h-4 w-4 ${isLoading ? "animate-spin" : ""}`} />
            Reload
          </Button>
          <Button
            size="sm"
            onClick={() => {
              setEditingRule(null);
              setIsRuleDialogOpen(true);
            }}
          >
            <Plus className="mr-2 h-4 w-4" />
            New Rule
          </Button>
        </div>
      </div>

      <SummaryCards summary={summary} isLoading={isLoading} />

      <Separator />

      <div className="flex gap-2 border-b">
        {[
          { id: "rules", label: "Issue Rules", count: rules.length },
          { id: "extractors", label: "Extractors", count: extractors.length },
          { id: "checks", label: "Audit Checks", count: checks.length },
        ].map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id as typeof activeTab)}
            className={`border-b-2 px-4 py-2 text-sm font-medium transition-colors ${
              activeTab === tab.id
                ? "border-primary text-foreground"
                : "border-transparent text-muted-foreground hover:text-foreground"
            }`}
          >
            {tab.label}
            <Badge variant="secondary" className="ml-2">
              {tab.count}
            </Badge>
          </button>
        ))}
      </div>

      {activeTab === "rules" && (
        <div className="space-y-4">
          <div className="flex flex-wrap gap-3">
            <div className="relative min-w-[200px] flex-1">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                placeholder="Search rules..."
                value={searchQuery}
                onChange={(event) => setSearchQuery(event.target.value)}
                className="pl-9"
              />
            </div>
            <Select value={categoryFilter} onValueChange={setCategoryFilter}>
              <SelectTrigger className="w-[140px]">
                <SelectValue placeholder="Category" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Categories</SelectItem>
                {availableRuleCategories.map((category) => (
                  <SelectItem key={category} value={category}>
                    {category.charAt(0).toUpperCase() + category.slice(1)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Select value={severityFilter} onValueChange={setSeverityFilter}>
              <SelectTrigger className="w-[120px]">
                <SelectValue placeholder="Severity" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All</SelectItem>
                <SelectItem value="critical">Critical</SelectItem>
                <SelectItem value="warning">Warning</SelectItem>
                <SelectItem value="info">Info</SelectItem>
              </SelectContent>
            </Select>
            <Button
              variant={showOnlyCustom ? "secondary" : "outline"}
              size="sm"
              onClick={() => setShowOnlyCustom(!showOnlyCustom)}
            >
              {showOnlyCustom ? "Custom Only" : "All Rules"}
            </Button>
          </div>

          <div className="space-y-2">
            {isLoading ? (
              [...Array(5)].map((_, index) => <Skeleton key={index} className="h-16 w-full" />)
            ) : sortedRules.length === 0 ? (
              <div className="py-8 text-center text-muted-foreground">
                <AlertTriangle className="mx-auto mb-2 h-8 w-8 opacity-50" />
                <p>No rules found</p>
                {(searchQuery ||
                  categoryFilter !== "all" ||
                  severityFilter !== "all" ||
                  showOnlyCustom) && (
                  <Button
                    variant="link"
                    size="sm"
                    onClick={() => {
                      setSearchQuery("");
                      setCategoryFilter("all");
                      setSeverityFilter("all");
                      setShowOnlyCustom(false);
                    }}
                  >
                    Clear filters
                  </Button>
                )}
              </div>
            ) : (
              sortedRules.map((rule) => (
                <RuleListItem
                  key={rule.id}
                  rule={rule}
                  onToggle={handleToggleRule}
                  onEdit={(selectedRule) => {
                    setEditingRule(selectedRule);
                    setIsRuleDialogOpen(true);
                  }}
                  onDelete={setDeletingRuleId}
                />
              ))
            )}
          </div>
        </div>
      )}

      {activeTab === "extractors" && (
        <div className="space-y-4">
          <div className="flex justify-end">
            <Button
              size="sm"
              onClick={() => {
                setEditingExtractor(null);
                setIsExtractorDialogOpen(true);
              }}
            >
              <Plus className="mr-2 h-4 w-4" />
              New Extractor
            </Button>
          </div>
          <div className="space-y-2">
            {isLoading ? (
              [...Array(3)].map((_, index) => <Skeleton key={index} className="h-16 w-full" />)
            ) : extractorConfigs.length === 0 ? (
              <div className="py-8 text-center text-muted-foreground">
                <Database className="mx-auto mb-2 h-8 w-8 opacity-50" />
                <p>No data extractors configured</p>
              </div>
            ) : (
              extractorConfigs.map((extractor) => {
                const preset = parseExtractorRulePreset(extractor.post_process);
                const thresholdSummary = formatPresetThreshold(
                  preset.default_rule_threshold_min,
                  preset.default_rule_threshold_max,
                );
                const hasPreset =
                  Boolean(preset.default_rule_severity) ||
                  Boolean(preset.default_rule_recommendation) ||
                  Boolean(thresholdSummary);

                return (
                  <div
                    key={extractor.id}
                    className={`flex items-center justify-between rounded-lg border p-3 transition-colors ${
                      extractor.is_enabled ? "bg-card/50" : "bg-muted/30 opacity-60"
                    }`}
                  >
                    <div className="flex min-w-0 flex-1 items-center gap-3">
                      <button
                        onClick={() => handleToggleExtractor(extractor.id, !extractor.is_enabled)}
                        className="flex-shrink-0"
                      >
                        {extractor.is_enabled ? (
                          <ToggleRight className="h-5 w-5 text-green-500" />
                        ) : (
                          <ToggleLeft className="h-5 w-5 text-muted-foreground" />
                        )}
                      </button>
                      <div className="min-w-0 flex-1">
                        <div className="flex items-center gap-2">
                          <p className="truncate font-medium">{extractor.display_name}</p>
                          {extractor.is_builtin && (
                            <Badge variant="outline" className="text-xs">
                              Built-in
                            </Badge>
                          )}
                        </div>
                        <div className="mt-1 flex items-center gap-2">
                          <Badge variant="outline">{extractor.extractor_type}</Badge>
                          <span className="font-mono text-xs text-muted-foreground">
                            {extractor.selector}
                          </span>
                          {extractor.attribute && (
                            <span className="text-xs text-muted-foreground">
                              @{extractor.attribute}
                            </span>
                          )}
                        </div>
                        {extractor.description && (
                          <p className="mt-1 truncate text-xs text-muted-foreground">
                            {extractor.description}
                          </p>
                        )}

                        {hasPreset && (
                          <div className="mt-2 flex flex-wrap items-center gap-2">
                            <span className="text-xs text-muted-foreground">Default rule:</span>
                            {preset.default_rule_severity && (
                              <Badge variant="secondary" className="text-[10px] capitalize">
                                {preset.default_rule_severity}
                              </Badge>
                            )}
                            {thresholdSummary && (
                              <Badge variant="outline" className="text-[10px]">
                                Threshold {thresholdSummary}
                              </Badge>
                            )}
                            {preset.default_rule_recommendation && (
                              <span className="max-w-[420px] truncate text-xs text-muted-foreground">
                                {preset.default_rule_recommendation}
                              </span>
                            )}
                          </div>
                        )}
                      </div>
                    </div>
                    <div className="flex items-center gap-1">
                      {!extractor.is_builtin && (
                        <>
                          <Button
                            variant="ghost"
                            size="icon"
                            className="h-8 w-8"
                            onClick={() => {
                              setEditingExtractor(extractor);
                              setIsExtractorDialogOpen(true);
                            }}
                          >
                            <Edit className="h-4 w-4" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="icon"
                            className="h-8 w-8 text-destructive hover:text-destructive"
                            onClick={() => setDeletingExtractorId(extractor.id)}
                          >
                            <Trash2 className="h-4 w-4" />
                          </Button>
                        </>
                      )}
                    </div>
                  </div>
                );
              })
            )}
          </div>
        </div>
      )}

      {activeTab === "checks" && (
        <div className="space-y-2">
          {isLoading ? (
            [...Array(3)].map((_, index) => <Skeleton key={index} className="h-16 w-full" />)
          ) : checks.length === 0 ? (
            <div className="py-8 text-center text-muted-foreground">
              <CheckCircle2 className="mx-auto mb-2 h-8 w-8 opacity-50" />
              <p>No audit checks configured</p>
            </div>
          ) : (
            checks.map((check) => (
              <div
                key={check.key}
                className="flex items-center justify-between rounded-lg border bg-card/50 p-3"
              >
                <div>
                  <p className="font-medium">{check.label}</p>
                  <div className="mt-1 flex items-center gap-2">
                    <Badge variant="outline">{check.category}</Badge>
                    <span className="text-xs text-muted-foreground">Weight: {check.weight}</span>
                    {check.is_builtin && <Badge variant="secondary">Built-in</Badge>}
                  </div>
                </div>
              </div>
            ))
          )}
        </div>
      )}

      <EnhancedRuleDialog
        key={`${editingRule?.id ?? "new"}:${isRuleDialogOpen ? "open" : "closed"}`}
        open={isRuleDialogOpen}
        onOpenChange={setIsRuleDialogOpen}
        rule={editingRule}
        onCreate={handleCreateRule}
        onUpdate={handleUpdateRule}
      />

      <AlertDialog open={!!deletingRuleId} onOpenChange={() => setDeletingRuleId(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Rule</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete this custom rule? This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleDeleteRule}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <Dialog open={isExtractorDialogOpen} onOpenChange={setIsExtractorDialogOpen}>
        <DialogContent className="flex max-h-[90vh] flex-col overflow-hidden sm:max-w-[560px]">
          <DialogHeader>
            <DialogTitle>
              {editingExtractor ? "Edit Extractor" : "Create Custom Extractor"}
            </DialogTitle>
            <DialogDescription>
              {editingExtractor
                ? "Modify the settings for this custom extractor."
                : "Define a new custom data extractor to extract information from pages."}
            </DialogDescription>
          </DialogHeader>
          <div className="flex-1 overflow-y-auto pr-1">
            <ExtractorDialogContent
              extractor={editingExtractor}
              onCreate={handleCreateExtractor}
              onUpdate={handleUpdateExtractor}
              onCancel={() => setIsExtractorDialogOpen(false)}
            />
          </div>
        </DialogContent>
      </Dialog>

      <AlertDialog open={!!deletingExtractorId} onOpenChange={() => setDeletingExtractorId(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Extractor</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete this custom extractor? This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleDeleteExtractor}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
