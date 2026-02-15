// Import generated bindings for internal type references and re-export commonly used types
import type {
  AnalysisResults,
  PageAnalysisData,
  CompleteAnalysisResponse,
  SeoIssue,
  IssueSeverity,
  ImageElement,
  JsonValue,
  AnalysisProgress,
  AnalysisSettingsRequest,
  AnalysisJobResponse,
  LinkDetail
} from "../bindings"

// Re-export selected generated types as a single canonical surface for the app
export type {
  AnalysisResults,
  PageAnalysisData,
  CompleteAnalysisResponse,
  SeoIssue,
  IssueSeverity,
  ImageElement,
  JsonValue,
  AnalysisProgress,
  AnalysisSettingsRequest,
  AnalysisJobResponse,
  LinkDetail
}

// Backwards-compatible aliases for older names used across the app
export type CompleteAnalysisResult = CompleteAnalysisResponse


// Detailed Lighthouse audit result for a single check
export interface LighthouseAuditResult {
  passed: boolean
  value: string | null
  score: number
  description?: string
}

// SEO-specific Lighthouse audits
export interface LighthouseSeoAudits {
  document_title: LighthouseAuditResult
  meta_description: LighthouseAuditResult
  viewport: LighthouseAuditResult
  canonical: LighthouseAuditResult
  hreflang: LighthouseAuditResult
  robots_txt: LighthouseAuditResult
  crawlable_anchors: LighthouseAuditResult
  link_text: LighthouseAuditResult
  image_alt: LighthouseAuditResult
  http_status_code: LighthouseAuditResult
  is_crawlable: LighthouseAuditResult
}

// Core Web Vitals and performance metrics
export interface LighthousePerformanceMetrics {
  first_contentful_paint: number | null
  largest_contentful_paint: number | null
  speed_index: number | null
  time_to_interactive: number | null
  total_blocking_time: number | null
  cumulative_layout_shift: number | null
}



// Use generated type for issues
export type IssueType = IssueSeverity


// Re-exported as alias at the top: CompleteAnalysisResult = CompleteAnalysisResponse

// ============================================================================
// Extended types for detailed page view (backend will populate these)
// ============================================================================

export type HeadingTag = 'h1' | 'h2' | 'h3' | 'h4' | 'h5' | 'h6'

export interface HeadingElement {
  tag: HeadingTag | string
  text: string
}

// ImageElement is provided by generated bindings; don't duplicate it here.

export interface LinkElement {
  href: string
  text: string
  is_external: boolean
  status_code: number | null
}

// Extended page data with detailed elements
export type PageDetailData = PageAnalysisData
