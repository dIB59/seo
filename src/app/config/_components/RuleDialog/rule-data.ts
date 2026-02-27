import type { LucideIcon } from "lucide-react";
import {
  FileText,
  Type,
  Heading1,
  Link,
  Smartphone,
  Image,
  File,
  Lock,
  Gauge,
  Hash,
  Settings2,
  Eye,
  CheckCircle,
  AlertTriangle,
  InfoIcon,
} from "lucide-react";
import type { ExtensionCategory, RuleSeverity, RuleType } from "@/src/api/extensions";

// ============================================================================
// Types
// ============================================================================

export interface RuleTemplate {
  id: string;
  name: string;
  description: string;
  category: ExtensionCategory;
  ruleType: RuleType;
  targetField: string;
  thresholdMin?: string;
  thresholdMax?: string;
  regexPattern?: string;
  recommendation: string;
  severity: RuleSeverity;
  icon: LucideIcon;
}

export interface TargetField {
  value: string;
  label: string;
  description: string;
}

export interface CategoryConfig {
  label: string;
  accent: string;
  lightBg: string;
}

export interface RuleTypeConfig {
  label: string;
  description: string;
  icon: LucideIcon;
}

// ============================================================================
// Rule Templates
// ============================================================================

export const RULE_TEMPLATES: RuleTemplate[] = [
  {
    id: "meta-description-length",
    name: "Meta Description Length",
    description: "Check if meta description is within optimal range (70-160 characters)",
    category: "seo",
    ruleType: "threshold",
    targetField: "meta_description",
    thresholdMin: "70",
    thresholdMax: "160",
    recommendation:
      "Write a meta description between 70-160 characters that includes your primary keyword and clearly summarizes the page content.",
    severity: "warning",
    icon: FileText,
  },
  {
    id: "title-length",
    name: "Title Tag Length",
    description: "Ensure title tag is within optimal range (30-60 characters)",
    category: "seo",
    ruleType: "threshold",
    targetField: "title",
    thresholdMin: "30",
    thresholdMax: "60",
    recommendation:
      "Create a title tag between 30-60 characters that includes your primary keyword near the beginning.",
    severity: "warning",
    icon: Type,
  },
  {
    id: "h1-count",
    name: "Single H1 Heading",
    description: "Verify that each page has exactly one H1 heading",
    category: "seo",
    ruleType: "threshold",
    targetField: "h1_count",
    thresholdMin: "1",
    thresholdMax: "1",
    recommendation:
      "Use exactly one H1 heading per page that includes your main topic or primary keyword.",
    severity: "warning",
    icon: Heading1,
  },
  {
    id: "canonical-url",
    name: "Canonical URL Presence",
    description: "Check if canonical URL is defined",
    category: "seo",
    ruleType: "presence",
    targetField: "canonical_url",
    recommendation: "Add a canonical URL to your page to prevent duplicate content issues.",
    severity: "info",
    icon: Link,
  },
  {
    id: "viewport-meta",
    name: "Viewport Meta Tag",
    description: "Ensure viewport meta tag exists for mobile responsiveness",
    category: "mobile",
    ruleType: "presence",
    targetField: "viewport",
    recommendation:
      'Add <meta name="viewport" content="width=device-width, initial-scale=1"> to enable proper mobile rendering.',
    severity: "critical",
    icon: Smartphone,
  },
  {
    id: "image-alt-text",
    name: "Image Alt Text",
    description: "Verify that images have descriptive alt text",
    category: "accessibility",
    ruleType: "presence",
    targetField: "alt_text",
    recommendation:
      "Add descriptive alt text to all images that explains the image content and purpose.",
    severity: "warning",
    icon: Image,
  },
  {
    id: "min-word-count",
    name: "Minimum Content Length",
    description: "Ensure pages have sufficient content (minimum 300 words)",
    category: "content",
    ruleType: "threshold",
    targetField: "word_count",
    thresholdMin: "300",
    recommendation:
      "Add more content to your page. Aim for at least 300 words of high-quality, relevant content.",
    severity: "info",
    icon: File,
  },
  {
    id: "https-check",
    name: "HTTPS Security",
    description: "Verify that site uses HTTPS protocol",
    category: "security",
    ruleType: "presence",
    targetField: "https",
    recommendation:
      "Ensure your website uses HTTPS. Obtain an SSL certificate and redirect HTTP to HTTPS.",
    severity: "critical",
    icon: Lock,
  },
  {
    id: "href-validation",
    name: "Valid HREF Attributes",
    description: "Check that anchor tags have valid href attributes",
    category: "technical",
    ruleType: "presence",
    targetField: "href",
    recommendation:
      "Ensure all anchor (<a>) tags have valid href attributes. Links without href are not clickable.",
    severity: "warning",
    icon: Link,
  },
];

// ============================================================================
// Configuration
// ============================================================================

export const CATEGORY_CONFIG: Record<string, CategoryConfig> = {
  seo: { label: "SEO", accent: "text-chart-1", lightBg: "bg-chart-1/10" },
  accessibility: { label: "Accessibility", accent: "text-chart-2", lightBg: "bg-chart-2/10" },
  performance: { label: "Performance", accent: "text-chart-3", lightBg: "bg-chart-3/10" },
  security: { label: "Security", accent: "text-destructive", lightBg: "bg-destructive/10" },
  content: { label: "Content", accent: "text-chart-4", lightBg: "bg-chart-4/10" },
  technical: { label: "Technical", accent: "text-muted-foreground", lightBg: "bg-muted" },
  ux: { label: "UX", accent: "text-chart-5", lightBg: "bg-chart-5/10" },
  mobile: { label: "Mobile", accent: "text-chart-1", lightBg: "bg-chart-1/10" },
};

export const RULE_TYPE_CONFIG: Record<string, RuleTypeConfig> = {
  presence: {
    label: "Presence Check",
    description: "Verifies that a field exists on the page",
    icon: Eye,
  },
  threshold: {
    label: "Threshold Check",
    description: "Checks if a value is within acceptable bounds",
    icon: Gauge,
  },
  regex: { label: "Pattern Match", description: "Validates against a regex pattern", icon: Hash },
  custom: { label: "Custom Rule", description: "Create custom validation logic", icon: Settings2 },
};

export const TARGET_FIELDS: TargetField[] = [
  { value: "title", label: "Title Tag", description: "The <title> tag in the HTML head" },
  { value: "meta_description", label: "Meta Description", description: "The meta description tag" },
  { value: "h1_count", label: "H1 Count", description: "Number of H1 headings on the page" },
  { value: "word_count", label: "Word Count", description: "Total words in the page content" },
  { value: "load_time_ms", label: "Load Time (ms)", description: "Page load time in milliseconds" },
  { value: "image_count", label: "Image Count", description: "Number of images on the page" },
  { value: "link_count", label: "Link Count", description: "Number of links on the page" },
  { value: "canonical_url", label: "Canonical URL", description: "The canonical URL if set" },
  { value: "viewport", label: "Viewport Meta", description: "The viewport meta tag" },
  { value: "robots", label: "Robots Meta", description: "The robots meta directive" },
  { value: "alt_text", label: "Image Alt Text", description: "Alt text from images" },
  { value: "https", label: "HTTPS", description: "Whether the page uses HTTPS" },
  { value: "url", label: "URL", description: "The page URL" },
  { value: "href", label: "HREF Attribute", description: "HREF attributes in anchor tags" },
  { value: "__custom__", label: "Custom Field", description: "Enter any custom field name" },
];

export const CATEGORIES: ExtensionCategory[] = [
  "seo",
  "accessibility",
  "performance",
  "security",
  "content",
  "technical",
  "ux",
  "mobile",
];

export const SEVERITIES: RuleSeverity[] = ["critical", "warning", "info"];

export const RULE_TYPES: RuleType[] = ["presence", "threshold", "regex", "custom"];

export const CUSTOM_FIELD_VALUE = "__custom__";
