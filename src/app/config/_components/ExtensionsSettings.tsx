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
  Filter,
  ChevronDown,
  ToggleLeft,
  ToggleRight,
  X,
  Save,
} from "lucide-react";
import { Button } from "@/src/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/src/components/ui/card";
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
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/src/components/ui/dialog";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/src/components/ui/dropdown-menu";
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
import { Label } from "@/src/components/ui/label";
import { Textarea } from "@/src/components/ui/textarea";
import { toast } from "sonner";
import {
  getExtensionSummary,
  getAllIssueRules,
  getAllExtractors,
  getAllAuditChecks,
  createCustomRule,
  updateCustomRule,
  deleteCustomRule,
  toggleRuleEnabled,
  reloadExtensions,
  filterRules,
  sortRules,
  type ExtensionSummary,
  type IssueRuleInfo,
  type DataExtractorInfo,
  type AuditCheckInfo,
  type CreateRuleRequest,
  type UpdateRuleRequest,
  type RuleSeverity,
  type ExtensionCategory,
  type RuleType,
} from "@/src/api/extensions";

// ============================================================================
// Summary Cards
// ============================================================================

interface SummaryCardsProps {
  summary: ExtensionSummary | null;
  isLoading: boolean;
}

function SummaryCards({ summary, isLoading }: SummaryCardsProps) {
  if (isLoading) {
    return (
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {[1, 2, 3].map((i) => (
          <Card key={i} className="bg-card/50">
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
      value: summary?.total_rules ?? 0,
      description: `${summary?.builtin_rules ?? 0} built-in, ${summary?.custom_rules ?? 0} custom`,
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
      value: summary?.total_checks ?? 0,
      description: "Scoring checks",
      icon: CheckCircle2,
      color: "text-green-500",
    },
  ];

  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
      {cards.map((card) => (
        <Card key={card.title} className="bg-card/50 hover:bg-card/70 transition-colors">
          <CardContent className="p-4">
            <div className="flex items-center gap-3">
              <div className={`p-2 rounded-lg bg-muted ${card.color}`}>
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

// ============================================================================
// Rule List Item
// ============================================================================

interface RuleListItemProps {
  rule: IssueRuleInfo;
  onToggle: (id: string, enabled: boolean) => void;
  onEdit: (rule: IssueRuleInfo) => void;
  onDelete: (id: string) => void;
}

function RuleListItem({ rule, onToggle, onEdit, onDelete }: RuleListItemProps) {
  const severityColors: Record<RuleSeverity, string> = {
    critical: "bg-red-500/10 text-red-500 border-red-500/20",
    warning: "bg-yellow-500/10 text-yellow-500 border-yellow-500/20",
    info: "bg-blue-500/10 text-blue-500 border-blue-500/20",
  };

  const severity = rule.severity as RuleSeverity;
  const ruleType = rule.rule_type as RuleType;

  return (
    <div
      className={`flex items-center justify-between p-3 rounded-lg border transition-colors ${
        rule.is_enabled ? "bg-card/50" : "bg-muted/30 opacity-60"
      }`}
    >
      <div className="flex items-center gap-3 flex-1 min-w-0">
        <button onClick={() => onToggle(rule.id, !rule.is_enabled)} className="flex-shrink-0">
          {rule.is_enabled ? (
            <ToggleRight className="h-5 w-5 text-green-500" />
          ) : (
            <ToggleLeft className="h-5 w-5 text-muted-foreground" />
          )}
        </button>
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <p className="font-medium truncate">{rule.name}</p>
            {rule.is_builtin && (
              <Badge variant="outline" className="text-xs">
                Built-in
              </Badge>
            )}
          </div>
          <div className="flex items-center gap-2 mt-1">
            <Badge variant="outline" className={severityColors[severity]}>
              {rule.severity}
            </Badge>
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

// ============================================================================
// Create/Edit Rule Dialog
// ============================================================================

interface RuleDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  rule: IssueRuleInfo | null; // null for create, existing for edit
  onCreate: (data: CreateRuleRequest) => Promise<void>;
  onUpdate: (data: UpdateRuleRequest) => Promise<void>;
}

function RuleDialog({ open, onOpenChange, rule, onCreate, onUpdate }: RuleDialogProps) {
  const [name, setName] = useState("");
  const [category, setCategory] = useState<ExtensionCategory>("seo");
  const [severity, setSeverity] = useState<RuleSeverity>("warning");
  const [ruleType, setRuleType] = useState<RuleType>("presence");
  const [targetField, setTargetField] = useState("");
  const [thresholdMin, setThresholdMin] = useState<string>("");
  const [thresholdMax, setThresholdMax] = useState<string>("");
  const [regexPattern, setRegexPattern] = useState("");
  const [recommendation, setRecommendation] = useState("");
  const [isSaving, setIsSaving] = useState(false);

  const isEditing = rule !== null;

  // Reset form when dialog opens
  useEffect(() => {
    if (open) {
      if (rule) {
        setName(rule.name);
        setCategory(rule.category as ExtensionCategory);
        setSeverity(rule.severity as RuleSeverity);
        setRuleType(rule.rule_type as RuleType);
        setTargetField(rule.target_field || "");
        setRecommendation(rule.recommendation || "");
      } else {
        setName("");
        setCategory("seo");
        setSeverity("warning");
        setRuleType("presence");
        setTargetField("");
        setThresholdMin("");
        setThresholdMax("");
        setRegexPattern("");
        setRecommendation("");
      }
    }
  }, [open, rule]);

  const handleSave = async () => {
    if (!name.trim() || (!isEditing && !targetField.trim())) {
      toast.error("Name and target field are required");
      return;
    }

    setIsSaving(true);
    try {
      if (isEditing && rule) {
        await onUpdate({
          id: rule.id,
          name,
          severity,
          threshold_min: thresholdMin ? parseFloat(thresholdMin) : undefined,
          threshold_max: thresholdMax ? parseFloat(thresholdMax) : undefined,
          regex_pattern: regexPattern || undefined,
          recommendation: recommendation || undefined,
        });
      } else {
        await onCreate({
          name,
          category,
          severity,
          rule_type: ruleType,
          target_field: targetField,
          threshold_min: thresholdMin ? parseFloat(thresholdMin) : undefined,
          threshold_max: thresholdMax ? parseFloat(thresholdMax) : undefined,
          regex_pattern: regexPattern || undefined,
          recommendation: recommendation || undefined,
        });
      }
      onOpenChange(false);
    } catch (error) {
      console.error("Failed to save rule:", error);
    } finally {
      setIsSaving(false);
    }
  };

  const categories: ExtensionCategory[] = [
    "seo",
    "accessibility",
    "performance",
    "security",
    "content",
    "technical",
    "ux",
    "mobile",
  ];
  const severities: RuleSeverity[] = ["critical", "warning", "info"];
  const ruleTypes: RuleType[] = ["presence", "threshold", "regex", "custom"];

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>{isEditing ? "Edit Rule" : "Create Custom Rule"}</DialogTitle>
          <DialogDescription>
            {isEditing
              ? "Modify the settings for this custom rule."
              : "Define a new custom issue rule for SEO analysis."}
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          <div className="grid gap-2">
            <Label htmlFor="name">Rule Name</Label>
            <Input
              id="name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g., Missing Meta Description"
            />
          </div>

          {!isEditing && (
            <>
              <div className="grid grid-cols-2 gap-4">
                <div className="grid gap-2">
                  <Label>Category</Label>
                  <Select
                    value={category}
                    onValueChange={(v) => setCategory(v as ExtensionCategory)}
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {categories.map((cat) => (
                        <SelectItem key={cat} value={cat}>
                          {cat.charAt(0).toUpperCase() + cat.slice(1)}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div className="grid gap-2">
                  <Label>Rule Type</Label>
                  <Select value={ruleType} onValueChange={(v) => setRuleType(v as RuleType)}>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {ruleTypes.map((type) => (
                        <SelectItem key={type} value={type}>
                          {type.charAt(0).toUpperCase() + type.slice(1)}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              </div>

              <div className="grid gap-2">
                <Label htmlFor="targetField">Target Field</Label>
                <Input
                  id="targetField"
                  value={targetField}
                  onChange={(e) => setTargetField(e.target.value)}
                  placeholder="e.g., meta_description"
                />
                <p className="text-xs text-muted-foreground">
                  The page data field this rule will check.
                </p>
              </div>
            </>
          )}

          <div className="grid gap-2">
            <Label>Severity</Label>
            <Select value={severity} onValueChange={(v) => setSeverity(v as RuleSeverity)}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {severities.map((sev) => (
                  <SelectItem key={sev} value={sev}>
                    {sev.charAt(0).toUpperCase() + sev.slice(1)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {ruleType === "threshold" && (
            <div className="grid grid-cols-2 gap-4">
              <div className="grid gap-2">
                <Label>Min Threshold</Label>
                <Input
                  type="number"
                  value={thresholdMin}
                  onChange={(e) => setThresholdMin(e.target.value)}
                  placeholder="e.g., 50"
                />
              </div>
              <div className="grid gap-2">
                <Label>Max Threshold</Label>
                <Input
                  type="number"
                  value={thresholdMax}
                  onChange={(e) => setThresholdMax(e.target.value)}
                  placeholder="e.g., 160"
                />
              </div>
            </div>
          )}

          {ruleType === "regex" && (
            <div className="grid gap-2">
              <Label>Regex Pattern</Label>
              <Input
                value={regexPattern}
                onChange={(e) => setRegexPattern(e.target.value)}
                placeholder="e.g., ^https://"
              />
            </div>
          )}

          <div className="grid gap-2">
            <Label>Recommendation</Label>
            <Textarea
              value={recommendation}
              onChange={(e) => setRecommendation(e.target.value)}
              placeholder="How to fix this issue..."
              rows={3}
            />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={handleSave} disabled={isSaving}>
            {isSaving ? (
              <>
                <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
                Saving...
              </>
            ) : (
              <>
                <Save className="h-4 w-4 mr-2" />
                {isEditing ? "Update Rule" : "Create Rule"}
              </>
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

// ============================================================================
// Main Extensions Settings Component
// ============================================================================

export function ExtensionsSettings() {
  // State
  const [summary, setSummary] = useState<ExtensionSummary | null>(null);
  const [rules, setRules] = useState<IssueRuleInfo[]>([]);
  const [extractors, setExtractors] = useState<DataExtractorInfo[]>([]);
  const [checks, setChecks] = useState<AuditCheckInfo[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<"rules" | "extractors" | "checks">("rules");

  // Filter state
  const [searchQuery, setSearchQuery] = useState("");
  const [categoryFilter, setCategoryFilter] = useState<string>("all");
  const [severityFilter, setSeverityFilter] = useState<string>("all");
  const [showOnlyCustom, setShowOnlyCustom] = useState(false);

  // Dialog state
  const [isRuleDialogOpen, setIsRuleDialogOpen] = useState(false);
  const [editingRule, setEditingRule] = useState<IssueRuleInfo | null>(null);
  const [deletingRuleId, setDeletingRuleId] = useState<string | null>(null);

  // Load data
  const loadData = useCallback(async () => {
    setIsLoading(true);
    try {
      const [summaryRes, rulesRes, extractorsRes, checksRes] = await Promise.all([
        getExtensionSummary(),
        getAllIssueRules(),
        getAllExtractors(),
        getAllAuditChecks(),
      ]);

      if (summaryRes.isOk()) setSummary(summaryRes.unwrap());
      if (rulesRes.isOk()) setRules(rulesRes.unwrap());
      if (extractorsRes.isOk()) setExtractors(extractorsRes.unwrap());
      if (checksRes.isOk()) setChecks(checksRes.unwrap());
    } catch (error) {
      console.error("Failed to load extensions:", error);
      toast.error("Failed to load extension data");
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    loadData();
  }, [loadData]);

  // Handlers
  const handleToggleRule = async (id: string, enabled: boolean) => {
    const result = await toggleRuleEnabled(id, enabled);
    if (result.isOk()) {
      setRules((prev) => prev.map((r) => (r.id === id ? { ...r, is_enabled: enabled } : r)));
      toast.success(enabled ? "Rule enabled" : "Rule disabled");
    } else {
      toast.error("Failed to toggle rule");
    }
  };

  const handleCreateRule = async (data: CreateRuleRequest) => {
    const result = await createCustomRule(data);
    if (result.isOk()) {
      setRules((prev) => [...prev, result.unwrap()]);
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
      setRules((prev) => prev.map((r) => (r.id === data.id ? result.unwrap() : r)));
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
      setRules((prev) => prev.filter((r) => r.id !== deletingRuleId));
      toast.success("Rule deleted successfully");
    } else {
      toast.error("Failed to delete rule");
    }
    setDeletingRuleId(null);
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

  // Filtered rules
  const filteredRules = filterRules(rules, {
    category: categoryFilter !== "all" ? categoryFilter : undefined,
    severity: severityFilter !== "all" ? severityFilter : undefined,
    is_builtin: showOnlyCustom ? false : undefined,
    search: searchQuery || undefined,
  });

  const sortedRules = sortRules(filteredRules, "name");

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-semibold flex items-center gap-2">
            <Puzzle className="h-5 w-5" />
            Extension System
          </h3>
          <p className="text-sm text-muted-foreground">
            Manage issue rules, data extractors, and audit checks.
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={handleReload} disabled={isLoading}>
            <RefreshCw className={`h-4 w-4 mr-2 ${isLoading ? "animate-spin" : ""}`} />
            Reload
          </Button>
          <Button
            size="sm"
            onClick={() => {
              setEditingRule(null);
              setIsRuleDialogOpen(true);
            }}
          >
            <Plus className="h-4 w-4 mr-2" />
            New Rule
          </Button>
        </div>
      </div>

      {/* Summary Cards */}
      <SummaryCards summary={summary} isLoading={isLoading} />

      <Separator />

      {/* Tabs */}
      <div className="flex gap-2 border-b">
        {[
          { id: "rules", label: "Issue Rules", count: rules.length },
          { id: "extractors", label: "Extractors", count: extractors.length },
          { id: "checks", label: "Audit Checks", count: checks.length },
        ].map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id as typeof activeTab)}
            className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
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

      {/* Rules Tab */}
      {activeTab === "rules" && (
        <div className="space-y-4">
          {/* Filters */}
          <div className="flex flex-wrap gap-3">
            <div className="relative flex-1 min-w-[200px]">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder="Search rules..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="pl-9"
              />
            </div>
            <Select value={categoryFilter} onValueChange={setCategoryFilter}>
              <SelectTrigger className="w-[140px]">
                <SelectValue placeholder="Category" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Categories</SelectItem>
                {["seo", "accessibility", "performance", "security", "content", "technical"].map(
                  (cat) => (
                    <SelectItem key={cat} value={cat}>
                      {cat.charAt(0).toUpperCase() + cat.slice(1)}
                    </SelectItem>
                  ),
                )}
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

          {/* Rules List */}
          <div className="space-y-2">
            {isLoading ? (
              [...Array(5)].map((_, i) => <Skeleton key={i} className="h-16 w-full" />)
            ) : sortedRules.length === 0 ? (
              <div className="text-center py-8 text-muted-foreground">
                <AlertTriangle className="h-8 w-8 mx-auto mb-2 opacity-50" />
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
                  onEdit={(r) => {
                    setEditingRule(r);
                    setIsRuleDialogOpen(true);
                  }}
                  onDelete={setDeletingRuleId}
                />
              ))
            )}
          </div>
        </div>
      )}

      {/* Extractors Tab */}
      {activeTab === "extractors" && (
        <div className="space-y-2">
          {isLoading ? (
            [...Array(3)].map((_, i) => <Skeleton key={i} className="h-16 w-full" />)
          ) : extractors.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              <Database className="h-8 w-8 mx-auto mb-2 opacity-50" />
              <p>No data extractors configured</p>
            </div>
          ) : (
            extractors.map((extractor) => (
              <div
                key={extractor.id}
                className="flex items-center justify-between p-3 rounded-lg border bg-card/50"
              >
                <div>
                  <p className="font-medium">{extractor.name}</p>
                  <div className="flex items-center gap-2 mt-1">
                    <Badge variant="outline">{extractor.extractor_type}</Badge>
                    {extractor.is_builtin && <Badge variant="secondary">Built-in</Badge>}
                  </div>
                </div>
              </div>
            ))
          )}
        </div>
      )}

      {/* Checks Tab */}
      {activeTab === "checks" && (
        <div className="space-y-2">
          {isLoading ? (
            [...Array(3)].map((_, i) => <Skeleton key={i} className="h-16 w-full" />)
          ) : checks.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              <CheckCircle2 className="h-8 w-8 mx-auto mb-2 opacity-50" />
              <p>No audit checks configured</p>
            </div>
          ) : (
            checks.map((check) => (
              <div
                key={check.key}
                className="flex items-center justify-between p-3 rounded-lg border bg-card/50"
              >
                <div>
                  <p className="font-medium">{check.label}</p>
                  <div className="flex items-center gap-2 mt-1">
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

      {/* Create/Edit Rule Dialog */}
      <RuleDialog
        open={isRuleDialogOpen}
        onOpenChange={setIsRuleDialogOpen}
        rule={editingRule}
        onCreate={handleCreateRule}
        onUpdate={handleUpdateRule}
      />

      {/* Delete Confirmation Dialog */}
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
    </div>
  );
}
