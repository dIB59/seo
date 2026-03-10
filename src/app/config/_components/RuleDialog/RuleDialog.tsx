"use client";

import { useState, useMemo } from "react";
import useSWR from "swr";
import {
  Plus,
  Edit,
  RefreshCw,
  Search,
  ChevronRight,
  ChevronLeft,
  Save,
  HelpCircle,
  Wand2,
  Sparkles,
  AlertCircle,
  AlertTriangle,
  Info as InfoIcon,
} from "lucide-react";
import { Button } from "@/src/components/ui/button";
import { Input } from "@/src/components/ui/input";
import { Textarea } from "@/src/components/ui/textarea";
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
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/src/components/ui/tooltip";
import { ScrollArea } from "@/src/components/ui/scroll-area";
import { toast } from "sonner";
import {
  type CreateRuleRequest,
  type UpdateRuleRequest,
  type IssueRuleInfo,
  getRuleFieldRegistry,
} from "@/src/api/extensions";
import type { ExtensionCategory, RuleType } from "@/src/lib/types/extension";
import {
  RULE_TYPE_CONFIG,
  SEVERITIES,
  RULE_TYPES,
  CUSTOM_FIELD_VALUE,
  CUSTOM_CATEGORY_VALUE,
  type RuleTemplate,
  type TargetField,
} from "./rule-config";
import { Step, TemplateCard, PreviewPanel, ChangesTracker } from "./components";

interface RuleDialogFormState {
  name: string;
  category: ExtensionCategory | typeof CUSTOM_CATEGORY_VALUE;
  severity: "critical" | "warning" | "info";
  ruleType: RuleType;
  targetField: string;
  customTargetField: string;
  customCategory: string;
  thresholdMin: string;
  thresholdMax: string;
  regexPattern: string;
  recommendation: string;
  originalValues: {
    name?: string;
    severity?: string;
    recommendation?: string;
    thresholdMin?: number;
    thresholdMax?: number;
    regexPattern?: string;
  };
}

function createInitialFormState(rule: IssueRuleInfo | null): RuleDialogFormState {
  if (!rule) {
    return {
      name: "",
      category: "seo",
      severity: "warning",
      ruleType: "presence",
      targetField: "",
      customTargetField: "",
      customCategory: "",
      thresholdMin: "",
      thresholdMax: "",
      regexPattern: "",
      recommendation: "",
      originalValues: {},
    };
  }

  return {
    name: rule.name,
    category: rule.category as ExtensionCategory,
    severity: rule.severity as "critical" | "warning" | "info",
    ruleType: rule.rule_type as RuleType,
    targetField: rule.target_field || "",
    customTargetField: "",
    customCategory: "",
    thresholdMin: rule.threshold_min?.toString() || "",
    thresholdMax: rule.threshold_max?.toString() || "",
    regexPattern: rule.regex_pattern || "",
    recommendation: rule.recommendation || "",
    originalValues: {
      name: rule.name,
      severity: rule.severity,
      recommendation: rule.recommendation || "",
      thresholdMin: rule.threshold_min ?? undefined,
      thresholdMax: rule.threshold_max ?? undefined,
      regexPattern: rule.regex_pattern || "",
    },
  };
}

function toTitleCase(value: string): string {
  return value
    .replace(/[_-]+/g, " ")
    .replace(/\s+/g, " ")
    .trim()
    .replace(/\b\w/g, (char) => char.toUpperCase());
}

// ============================================================================
// Main Dialog Component
// ============================================================================

interface RuleDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  rule: IssueRuleInfo | null;
  onCreate: (data: CreateRuleRequest) => Promise<void>;
  onUpdate: (data: UpdateRuleRequest) => Promise<void>;
}

export function RuleDialog({ open, onOpenChange, rule, onCreate, onUpdate }: RuleDialogProps) {
  const initialFormState = useMemo(() => createInitialFormState(rule), [rule]);

  // Form state
  const [name, setName] = useState(initialFormState.name);
  const [category, setCategory] = useState<ExtensionCategory | typeof CUSTOM_CATEGORY_VALUE>(
    initialFormState.category,
  );
  const [severity, setSeverity] = useState<"critical" | "warning" | "info">(
    initialFormState.severity,
  );
  const [ruleType, setRuleType] = useState<RuleType>(initialFormState.ruleType);
  const [targetField, setTargetField] = useState(initialFormState.targetField);
  const [customTargetField, setCustomTargetField] = useState(initialFormState.customTargetField);
  const [customCategory, setCustomCategory] = useState(initialFormState.customCategory);
  const [thresholdMin, setThresholdMin] = useState(initialFormState.thresholdMin);
  const [thresholdMax, setThresholdMax] = useState(initialFormState.thresholdMax);
  const [regexPattern, setRegexPattern] = useState(initialFormState.regexPattern);
  const [recommendation, setRecommendation] = useState(initialFormState.recommendation);
  const [isSaving, setIsSaving] = useState(false);

  // UI state
  const [currentStep, setCurrentStep] = useState(0);
  const [completedSteps, setCompletedSteps] = useState<Set<number>>(new Set());
  const [searchTerm, setSearchTerm] = useState("");
  const [originalValues] = useState(initialFormState.originalValues);

  const { data: fieldRegistry = [] } = useSWR(
    open ? "rule-field-registry" : null,
    async () => {
      const result = await getRuleFieldRegistry();
      return result.isOk() ? result.unwrap() : [];
    },
    { revalidateOnFocus: false },
  );

  const categoryOptionMap = useMemo(() => {
    const entries = fieldRegistry
      .filter((field) => field.category_id?.trim())
      .map((field) => {
        const categoryId = field.category_id!.trim();
        return [categoryId, field.category_label?.trim() || toTitleCase(categoryId)] as const;
      });

    return new Map(entries);
  }, [fieldRegistry]);

  const categoryOptions = useMemo<(ExtensionCategory | typeof CUSTOM_CATEGORY_VALUE)[]>(() => {
    const values = [...categoryOptionMap.keys()].sort((left, right) => left.localeCompare(right));
    if (rule?.category && !values.includes(rule.category)) {
      values.unshift(rule.category);
    }
    return [
      ...values.filter((value): value is ExtensionCategory => value !== CUSTOM_CATEGORY_VALUE),
      CUSTOM_CATEGORY_VALUE,
    ];
  }, [categoryOptionMap, rule?.category]);

  const targetFieldOptions = useMemo(() => {
    const options: TargetField[] = [];
    const seen = new Set<string>();

    for (const field of fieldRegistry) {
      if (!seen.has(field.target_field)) {
        seen.add(field.target_field);
        options.push({
          value: field.target_field,
          label: field.kind === "category" ? `Category: ${field.label}` : `Field: ${field.label}`,
          description:
            field.description ||
            (field.kind === "category"
              ? "Any extracted values in this category"
              : `Rule-targetable field ${field.id}`),
        });
      }
    }

    if (rule?.target_field && !seen.has(rule.target_field)) {
      seen.add(rule.target_field);
      options.unshift({
        value: rule.target_field,
        label: `Current: ${rule.target_field}`,
        description: "Target field used by this existing rule",
      });
    }

    options.push({
      value: CUSTOM_FIELD_VALUE,
      label: "Custom Field",
      description: "Enter any custom target field value",
    });

    return options;
  }, [fieldRegistry, rule]);

  const dynamicTemplates = useMemo(() => {
    const templates: RuleTemplate[] = [];

    for (const field of fieldRegistry.filter((value) => value.kind === "extractor").slice(0, 8)) {
      const categoryId = (field.category_id || "technical") as ExtensionCategory;
      const target = field.target_field;
      const defaultSeverity =
        (field.default_rule_severity as "critical" | "warning" | "info") || "warning";
      const defaultRecommendation =
        field.default_rule_recommendation ||
        `Ensure content targeted by field "${field.label}" exists on the page.`;
      const defaultThresholdMin = field.default_rule_threshold_min;
      const defaultThresholdMax = field.default_rule_threshold_max;

      templates.push({
        id: `presence-${field.id}`,
        name: `${field.label} Presence`,
        description: `Validate that field "${field.label}" returns at least one value`,
        category: categoryId,
        ruleType: "presence",
        targetField: target,
        recommendation: defaultRecommendation,
        severity: defaultSeverity,
        icon: RULE_TYPE_CONFIG.presence.icon,
      });

      if (/count|length|score|time|size|ms|kb|mb/i.test(field.id + field.label)) {
        templates.push({
          id: `threshold-${field.id}`,
          name: `${field.label} Threshold`,
          description: `Check whether "${field.label}" stays within acceptable bounds`,
          category: categoryId,
          ruleType: "threshold",
          targetField: target,
          thresholdMin: defaultThresholdMin?.toString() || "1",
          thresholdMax: defaultThresholdMax?.toString(),
          recommendation: defaultRecommendation,
          severity: defaultSeverity,
          icon: RULE_TYPE_CONFIG.threshold.icon,
        });
      }
    }

    if (templates.length === 0) {
      templates.push({
        id: "custom-template",
        name: "Custom Extracted Field Rule",
        description: "Start with a custom extracted field and define your own validation logic",
        category: "technical",
        ruleType: "presence",
        targetField: CUSTOM_FIELD_VALUE,
        recommendation: "Select a target field and define how this rule should validate it.",
        severity: "warning",
        icon: RULE_TYPE_CONFIG.custom.icon,
      });
    }

    return templates;
  }, [fieldRegistry]);

  const isEditing = rule !== null;
  const steps = isEditing
    ? ["Basics", "Recommendation"]
    : ["Templates", "Basics", "Validation", "Recommendation"];

  // Handlers
  const handleTemplateSelect = (template: RuleTemplate) => {
    const selectedCategory = categoryOptions.includes(template.category)
      ? template.category
      : CUSTOM_CATEGORY_VALUE;

    setName(template.name);
    setCategory(selectedCategory);
    setCustomCategory(selectedCategory === CUSTOM_CATEGORY_VALUE ? template.category : "");
    setRuleType(template.ruleType);
    setTargetField(template.targetField);
    setSeverity(template.severity);
    setRecommendation(template.recommendation);
    setThresholdMin(template.thresholdMin || "");
    setThresholdMax(template.thresholdMax || "");
    setRegexPattern(template.regexPattern || "");
    markStepComplete(0);
    setCurrentStep(1);
  };

  const markStepComplete = (step: number) => {
    setCompletedSteps((previous) => new Set([...previous, step]));
  };

  const handleNext = () => {
    markStepComplete(currentStep);
    if (currentStep < steps.length - 1) {
      setCurrentStep(currentStep + 1);
    }
  };

  const handleBack = () => {
    if (currentStep > 0) {
      setCurrentStep(currentStep - 1);
    }
  };

  const handleSave = async () => {
    if (!name.trim()) {
      toast.error("Rule name is required");
      return;
    }

    if (!isEditing && !targetField.trim()) {
      toast.error("Target field is required");
      return;
    }

    if (!isEditing && targetField === CUSTOM_FIELD_VALUE && !customTargetField.trim()) {
      toast.error("Custom field name is required");
      return;
    }

    if (category === CUSTOM_CATEGORY_VALUE && !customCategory.trim()) {
      toast.error("Custom category is required");
      return;
    }

    setIsSaving(true);
    try {
      if (isEditing && rule) {
        await onUpdate({
          id: rule.id,
          name,
          severity,
          threshold_min: thresholdMin ? parseFloat(thresholdMin) : null,
          threshold_max: thresholdMax ? parseFloat(thresholdMax) : null,
          regex_pattern: regexPattern || null,
          recommendation: recommendation || null,
          is_enabled: null,
        });
      } else {
        await onCreate({
          name,
          category: category === CUSTOM_CATEGORY_VALUE ? customCategory : category,
          severity,
          rule_type: ruleType,
          target_field: targetField === CUSTOM_FIELD_VALUE ? customTargetField : targetField,
          threshold_min: thresholdMin ? parseFloat(thresholdMin) : null,
          threshold_max: thresholdMax ? parseFloat(thresholdMax) : null,
          regex_pattern: regexPattern || null,
          recommendation: recommendation || null,
          selector: null,
          attribute: null,
          multiple: null,
          min_count: null,
          max_count: null,
          min_length: null,
          max_length: null,
          expected_value: null,
          negate: null,
        });
      }
      onOpenChange(false);
    } catch (error) {
      console.error("Failed to save rule:", error);
      toast.error("Failed to save rule. Please try again.");
    } finally {
      setIsSaving(false);
    }
  };

  // Filter templates based on search
  const filteredTemplates = dynamicTemplates.filter(
    (t) =>
      t.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
      t.description.toLowerCase().includes(searchTerm.toLowerCase()) ||
      t.category.toLowerCase().includes(searchTerm.toLowerCase()),
  );

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[780px] max-h-[90vh] overflow-hidden flex flex-col p-0 gap-0">
        {/* Header */}
        <div className="px-6 py-5 border-b border-border/60 bg-gradient-to-br from-background to-muted/20">
          <DialogHeader className="space-y-1">
            <DialogTitle className="text-xl flex items-center gap-3">
              {isEditing ? (
                <>
                  <div className="p-2 rounded-lg bg-primary/10">
                    <Edit className="h-4 w-4 text-primary" />
                  </div>
                  <span className="bg-gradient-to-r from-foreground to-foreground/70 bg-clip-text text-transparent">
                    Edit Custom Rule
                  </span>
                </>
              ) : (
                <>
                  <div className="p-2 rounded-lg bg-primary/10">
                    <Plus className="h-4 w-4 text-primary" />
                  </div>
                  <span className="bg-gradient-to-r from-foreground to-foreground/70 bg-clip-text text-transparent">
                    Create Custom Rule
                  </span>
                </>
              )}
            </DialogTitle>
            <DialogDescription className="text-muted-foreground ml-11">
              {isEditing
                ? "Modify your custom rule. Changes apply to future analyses."
                : "Build a new validation rule for SEO analysis. Choose a template or start from scratch."}
            </DialogDescription>
          </DialogHeader>
        </div>

        {/* Progress */}
        <div className="px-6 py-4 border-b border-border/40 bg-muted/10">
          <div className="flex items-center justify-between">
            {steps.map((step, index) => (
              <Step
                key={step}
                number={index + 1}
                title={step}
                isActive={currentStep === index}
                isCompleted={completedSteps.has(index)}
              />
            ))}
          </div>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-hidden">
          <ScrollArea className="h-[50vh]">
            <div className="p-6">
              {/* Step 0: Templates */}
              {currentStep === 0 && !isEditing && (
                <div className="space-y-6">
                  <div className="text-center space-y-4 pb-4">
                    <div className="w-16 h-16 mx-auto rounded-2xl bg-gradient-to-br from-primary/10 to-primary/5 flex items-center justify-center border border-primary/20 shadow-lg shadow-primary/10">
                      <Wand2 className="h-7 w-7 text-primary" />
                    </div>
                    <div>
                      <h3 className="text-xl font-semibold bg-gradient-to-r from-foreground to-foreground/70 bg-clip-text text-transparent">
                        Choose a Starting Point
                      </h3>
                      <p className="text-sm text-muted-foreground mt-2 max-w-sm mx-auto">
                        Select from pre-built templates or create your own custom rule
                      </p>
                    </div>
                  </div>

                  <div className="relative">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                    <Input
                      placeholder="Search templates..."
                      value={searchTerm}
                      onChange={(e) => setSearchTerm(e.target.value)}
                      className="pl-10 h-11 bg-muted/50 border-border/60 focus:border-primary/50 focus:ring-primary/20"
                    />
                  </div>

                  <div className="grid grid-cols-1 gap-3">
                    {filteredTemplates.map((template) => (
                      <TemplateCard
                        key={template.id}
                        template={template}
                        onSelect={handleTemplateSelect}
                      />
                    ))}
                  </div>

                  <div className="pt-4 border-t border-border/40 text-center">
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => {
                        markStepComplete(0);
                        setCurrentStep(1);
                      }}
                      className="text-muted-foreground hover:text-foreground"
                    >
                      <Sparkles className="h-4 w-4 mr-2 text-primary" />
                      Start from scratch
                    </Button>
                  </div>
                </div>
              )}

              {/* Step 1: Basics (or only step for edit mode) */}
              {(currentStep === 1 || (currentStep === 0 && isEditing)) && (
                <div className="space-y-6">
                  {isEditing && (
                    <ChangesTracker
                      original={originalValues}
                      current={{
                        name,
                        severity,
                        recommendation,
                        thresholdMin,
                        thresholdMax,
                        regexPattern,
                      }}
                    />
                  )}

                  {/* Rule Name */}
                  <div className="space-y-2">
                    <div className="flex items-center gap-1.5">
                      <label className="text-sm font-semibold">Rule Name</label>
                      <span className="text-destructive">*</span>
                      <TooltipProvider>
                        <Tooltip>
                          <TooltipTrigger>
                            <HelpCircle className="h-3.5 w-3.5 text-muted-foreground" />
                          </TooltipTrigger>
                          <TooltipContent side="right" className="max-w-xs">
                            <p className="text-sm">
                              A clear, descriptive name for your rule that will appear in analysis
                              reports.
                            </p>
                          </TooltipContent>
                        </Tooltip>
                      </TooltipProvider>
                    </div>
                    <Input
                      value={name}
                      onChange={(e) => setName(e.target.value)}
                      placeholder="e.g., Check Meta Description Length"
                      className="h-10"
                    />
                  </div>

                  {/* Category */}
                  <div className="space-y-2">
                    <label className="text-sm font-semibold">Category</label>
                    <div className="grid grid-cols-4 gap-2">
                      {categoryOptions.map((cat) => (
                        <button
                          key={cat}
                          type="button"
                          onClick={() => setCategory(cat)}
                          className={`p-2.5 rounded-lg border text-center text-sm font-medium transition-all ${
                            category === cat
                              ? "border-foreground/20 bg-muted"
                              : "border-border hover:border-foreground/10"
                          }`}
                        >
                          {cat === CUSTOM_CATEGORY_VALUE
                            ? "Custom"
                            : categoryOptionMap.get(cat) || toTitleCase(cat)}
                        </button>
                      ))}
                    </div>
                    {category === CUSTOM_CATEGORY_VALUE && (
                      <Input
                        value={customCategory}
                        onChange={(e) => setCustomCategory(e.target.value)}
                        placeholder="e.g., commerce"
                        className="h-10 mt-2"
                      />
                    )}
                  </div>

                  {/* Rule Type */}
                  {!isEditing && (
                    <div className="space-y-2">
                      <label className="text-sm font-semibold">Rule Type</label>
                      <div className="grid grid-cols-2 gap-2">
                        {RULE_TYPES.map((type) => {
                          const config = RULE_TYPE_CONFIG[type];
                          return (
                            <button
                              key={type}
                              type="button"
                              onClick={() => setRuleType(type)}
                              className={`p-3 rounded-lg border text-left transition-all ${
                                ruleType === type
                                  ? "border-foreground/20 bg-muted"
                                  : "border-border hover:border-foreground/10"
                              }`}
                            >
                              <div className="flex items-center gap-2">
                                {config.icon && <config.icon className="h-4 w-4" />}
                                <div>
                                  <div className="font-medium text-sm">{config.label}</div>
                                </div>
                              </div>
                            </button>
                          );
                        })}
                      </div>
                    </div>
                  )}
                </div>
              )}

              {/* Step 2: Validation */}
              {currentStep === 2 && !isEditing && (
                <div className="space-y-6">
                  {/* Target Field */}
                  <div className="space-y-2">
                    <div className="flex items-center gap-1.5">
                      <label className="text-sm font-semibold">Target Field</label>
                      <span className="text-destructive">*</span>
                    </div>
                    <Select value={targetField} onValueChange={setTargetField}>
                      <SelectTrigger className="h-10">
                        <SelectValue placeholder="Select a field to check" />
                      </SelectTrigger>
                      <SelectContent>
                        {targetFieldOptions.map((field) => (
                          <SelectItem key={field.value} value={field.value}>
                            <div>
                              <div className="font-medium">{field.label}</div>
                              <div className="text-xs text-muted-foreground">
                                {field.description}
                              </div>
                            </div>
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                    {targetField === CUSTOM_FIELD_VALUE && (
                      <div className="mt-3">
                        <label className="text-xs text-muted-foreground mb-1.5 block">
                          Enter custom field name
                        </label>
                        <Input
                          value={customTargetField}
                          onChange={(e) => setCustomTargetField(e.target.value)}
                          placeholder="e.g., field:extractor:open_graph_title or field:category:open_graph"
                          className="h-10 font-mono text-sm"
                        />
                      </div>
                    )}
                  </div>

                  {/* Threshold */}
                  {ruleType === "threshold" && (
                    <div className="p-4 rounded-xl border border-border/40 bg-muted/30 space-y-4">
                      <label className="text-sm font-semibold">Threshold Values</label>
                      <div className="grid grid-cols-2 gap-4">
                        <div>
                          <label className="text-xs text-muted-foreground">Minimum</label>
                          <Input
                            type="number"
                            value={thresholdMin}
                            onChange={(e) => setThresholdMin(e.target.value)}
                            placeholder="e.g., 300"
                          />
                        </div>
                        <div>
                          <label className="text-xs text-muted-foreground">Maximum</label>
                          <Input
                            type="number"
                            value={thresholdMax}
                            onChange={(e) => setThresholdMax(e.target.value)}
                            placeholder="e.g., 3000"
                          />
                        </div>
                      </div>
                    </div>
                  )}

                  {/* Regex */}
                  {ruleType === "regex" && (
                    <div className="space-y-2">
                      <label className="text-sm font-semibold">Regex Pattern</label>
                      <Input
                        value={regexPattern}
                        onChange={(e) => setRegexPattern(e.target.value)}
                        placeholder="e.g., ^https?://[^\s]+$"
                        className="font-mono h-10"
                      />
                    </div>
                  )}

                  {/* Severity */}
                  <div className="space-y-2">
                    <label className="text-sm font-semibold">Issue Severity</label>
                    <div className="grid grid-cols-3 gap-2">
                      {SEVERITIES.map((sev) => (
                        <button
                          key={sev}
                          type="button"
                          onClick={() => setSeverity(sev)}
                          className={`p-3 rounded-lg border text-center text-sm font-medium transition-all ${
                            severity === sev
                              ? sev === "critical"
                                ? "border-destructive bg-destructive/10 text-destructive"
                                : sev === "warning"
                                  ? "border-warning bg-warning/10 text-warning"
                                  : "border-chart-1 bg-chart-1/10 text-chart-1"
                              : "border-border hover:border-foreground/10"
                          }`}
                        >
                          <span className="capitalize flex items-center justify-center gap-1.5">
                            {sev === "critical" && <AlertCircle className="h-3.5 w-3.5" />}
                            {sev === "warning" && <AlertTriangle className="h-3.5 w-3.5" />}
                            {sev === "info" && <InfoIcon className="h-3.5 w-3.5" />}
                            {sev}
                          </span>
                        </button>
                      ))}
                    </div>
                  </div>
                </div>
              )}

              {/* Step 3/Last: Recommendation */}
              {(currentStep === 3 || (currentStep === 1 && isEditing)) && (
                <div className="space-y-5">
                  <div className="space-y-2">
                    <div className="flex items-center gap-1.5">
                      <label className="text-sm font-semibold">Recommendation</label>
                      <TooltipProvider>
                        <Tooltip>
                          <TooltipTrigger>
                            <HelpCircle className="h-3.5 w-3.5 text-muted-foreground" />
                          </TooltipTrigger>
                          <TooltipContent side="right" className="max-w-xs">
                            <p className="text-sm">
                              Guidance shown to users on how to fix this issue when detected.
                            </p>
                          </TooltipContent>
                        </Tooltip>
                      </TooltipProvider>
                    </div>
                    <Textarea
                      value={recommendation}
                      onChange={(e) => setRecommendation(e.target.value)}
                      placeholder="e.g., Add a meta description between 70-160 characters..."
                      rows={3}
                    />
                  </div>

                  <PreviewPanel
                    name={name}
                    category={category}
                    severity={severity}
                    ruleType={ruleType}
                    recommendation={recommendation}
                    thresholdMin={thresholdMin}
                    thresholdMax={thresholdMax}
                    regexPattern={regexPattern}
                  />
                </div>
              )}

              {/* Navigation */}
              <div className="flex items-center justify-between pt-6 border-t border-border/40 mt-6">
                <div>
                  {currentStep > 0 && (
                    <Button variant="ghost" onClick={handleBack} className="gap-2">
                      <ChevronLeft className="h-4 w-4" />
                      Back
                    </Button>
                  )}
                </div>
                <div className="flex gap-2">
                  {!isEditing && currentStep === 0 && (
                    <Button
                      variant="outline"
                      onClick={() => {
                        markStepComplete(0);
                        setCurrentStep(1);
                      }}
                    >
                      Skip Templates
                    </Button>
                  )}
                  {currentStep < steps.length - 1 && !isEditing && (
                    <Button onClick={handleNext} className="gap-2">
                      Continue
                      <ChevronRight className="h-4 w-4" />
                    </Button>
                  )}
                </div>
              </div>
            </div>
          </ScrollArea>
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-border/60 bg-gradient-to-t from-muted/30 to-background flex items-center justify-end gap-3">
          <Button variant="outline" onClick={() => onOpenChange(false)} className="hover:bg-muted">
            Cancel
          </Button>
          <Button
            onClick={handleSave}
            disabled={isSaving || (currentStep === 0 && !isEditing)}
            className="gap-2"
          >
            {isSaving ? (
              <RefreshCw className="h-4 w-4 animate-spin" />
            ) : (
              <>
                <Save className="h-4 w-4" />
                {isEditing ? "Save Changes" : "Create Rule"}
              </>
            )}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}

export default RuleDialog;
