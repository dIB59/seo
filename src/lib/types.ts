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

export type CompleteAnalysisResult = CompleteAnalysisResponse


export interface LighthouseAuditResult {
  passed: boolean
  value: string | null
  score: number
  description?: string
}

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

export interface LighthousePerformanceMetrics {
  first_contentful_paint: number | null
  largest_contentful_paint: number | null
  speed_index: number | null
  time_to_interactive: number | null
  total_blocking_time: number | null
  cumulative_layout_shift: number | null
}



export type IssueType = IssueSeverity


// Re-exported as alias at the top: CompleteAnalysisResult = CompleteAnalysisResponse


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
