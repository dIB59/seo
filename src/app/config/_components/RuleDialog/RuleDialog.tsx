"use client";

import { useState, useEffect } from "react";
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
  createCustomRule,
  updateCustomRule,
  type CreateRuleRequest,
  type UpdateRuleRequest,
  type ExtensionCategory,
  type IssueRuleInfo,
} from "@/src/api/extensions";
import {
  RULE_TEMPLATES,
  RULE_TYPE_CONFIG,
  TARGET_FIELDS,
  CATEGORIES,
  SEVERITIES,
  RULE_TYPES,
  CUSTOM_FIELD_VALUE,
} from "./rule-data";
import { Step, TemplateCard, PreviewPanel, ChangesTracker } from "./components";

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
  // Form state
  const [name, setName] = useState("");
  const [category, setCategory] = useState<ExtensionCategory>("seo");
  const [severity, setSeverity] = useState<"critical" | "warning" | "info">("warning");
  const [ruleType, setRuleType] = useState<"presence" | "threshold" | "regex" | "custom">(
    "presence",
  );
  const [targetField, setTargetField] = useState("");
  const [customTargetField, setCustomTargetField] = useState("");
  const [thresholdMin, setThresholdMin] = useState("");
  const [thresholdMax, setThresholdMax] = useState("");
  const [regexPattern, setRegexPattern] = useState("");
  const [recommendation, setRecommendation] = useState("");
  const [isSaving, setIsSaving] = useState(false);

  // UI state
  const [currentStep, setCurrentStep] = useState(0);
  const [completedSteps, setCompletedSteps] = useState<Set<number>>(new Set());
  const [searchTerm, setSearchTerm] = useState("");
  const [originalValues, setOriginalValues] = useState<{
    name?: string;
    severity?: string;
    recommendation?: string;
    thresholdMin?: number;
    thresholdMax?: number;
    regexPattern?: string;
  }>({});

  const isEditing = rule !== null;
  const steps = isEditing
    ? ["Basics", "Recommendation"]
    : ["Templates", "Basics", "Validation", "Recommendation"];

  // Reset form when dialog opens/closes or rule changes
  useEffect(() => {
    if (open) {
      if (rule) {
        setName(rule.name);
        setCategory(rule.category as ExtensionCategory);
        setSeverity(rule.severity as "critical" | "warning" | "info");
        setRuleType(rule.rule_type as "presence" | "threshold" | "regex" | "custom");
        setTargetField(rule.target_field || "");
        setRecommendation(rule.recommendation || "");
        setThresholdMin(rule.threshold_min?.toString() || "");
        setThresholdMax(rule.threshold_max?.toString() || "");
        setRegexPattern(rule.regex_pattern || "");
        setCustomTargetField("");

        setOriginalValues({
          name: rule.name,
          severity: rule.severity,
          recommendation: rule.recommendation || "",
          thresholdMin: rule.threshold_min,
          thresholdMax: rule.threshold_max,
          regexPattern: rule.regex_pattern || "",
        });

        setCurrentStep(0);
      } else {
        setName("");
        setCategory("seo");
        setSeverity("warning");
        setRuleType("presence");
        setTargetField("");
        setCustomTargetField("");
        setThresholdMin("");
        setThresholdMax("");
        setRegexPattern("");
        setRecommendation("");
        setOriginalValues({});
        setCurrentStep(0);
      }
      setCompletedSteps(new Set());
    }
  }, [open, rule]);

  // Handlers
  const handleTemplateSelect = (template: (typeof RULE_TEMPLATES)[0]) => {
    setName(template.name);
    setCategory(template.category);
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
    setCompletedSteps(new Set([...completedSteps, step]));
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
          target_field: targetField === CUSTOM_FIELD_VALUE ? customTargetField : targetField,
          threshold_min: thresholdMin ? parseFloat(thresholdMin) : undefined,
          threshold_max: thresholdMax ? parseFloat(thresholdMax) : undefined,
          regex_pattern: regexPattern || undefined,
          recommendation: recommendation || undefined,
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
  const filteredTemplates = RULE_TEMPLATES.filter(
    (t) =>
      t.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
      t.description.toLowerCase().includes(searchTerm.toLowerCase()) ||
      t.category.toLowerCase().includes(searchTerm.toLowerCase()),
  );

  // Get the actual target field value for display
  const getTargetFieldValue = () => {
    if (targetField === CUSTOM_FIELD_VALUE) {
      return customTargetField || "Custom field";
    }
    const field = TARGET_FIELDS.find((f) => f.value === targetField);
    return field?.label || targetField;
  };

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
                      {CATEGORIES.map((cat) => {
                        const config = RULE_TYPE_CONFIG[cat];
                        return (
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
                            {cat.charAt(0).toUpperCase() + cat.slice(1)}
                          </button>
                        );
                      })}
                    </div>
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
                        {TARGET_FIELDS.map((field) => (
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
                          placeholder="e.g., href, src, class, data-id"
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
