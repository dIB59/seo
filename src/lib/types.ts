// Types matching the Tauri backend

export interface AnalysisJobResponse {
  job_id: string
  url: string
  status: string
}

export interface AnalysisSettingsRequest {
  max_pages: number
  include_external_links: boolean
  check_images: boolean
  mobile_analysis: boolean
  lighthouse_analysis: boolean
  delay_between_requests: number
}

export interface AnalysisProgress {
  job_id: string
  url: string
  job_status: string
  result_id: string | null
  analysis_status: string | null
  progress: number | null
  analyzed_pages: number | null
  total_pages: number | null
}

export interface AnalysisSummary {
  analysis_id: string
  seo_score: number
  avg_load_time: number
  total_words: number
  total_issues: number
}

export interface AnalysisResults {
  id: string
  url: string
  status: string
  progress: number
  total_pages: number
  analyzed_pages: number
  started_at: string | null
  completed_at: string | null
  sitemap_found: boolean
  robots_txt_found: boolean
  ssl_certificate: boolean
  created_at: string
}

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

export interface PageAnalysisData {
  analysis_id: string
  url: string
  title: string | null
  meta_description: string | null
  meta_keywords: string | null
  canonical_url: string | null
  h1_count: number
  h2_count: number
  h3_count: number
  word_count: number
  image_count: number
  images_without_alt: number
  internal_links: number
  external_links: number
  load_time: number
  status_code: number | null
  content_size: number
  mobile_friendly: boolean
  has_structured_data: boolean
  lighthouse_performance: number | null
  lighthouse_accessibility: number | null
  lighthouse_best_practices: number | null
  lighthouse_seo: number | null
  // Detailed Lighthouse breakdowns
  lighthouse_seo_audits?: LighthouseSeoAudits | null
  lighthouse_performance_metrics?: LighthousePerformanceMetrics | null
  detailed_links?: LinkElement[]
}

export type IssueType = "critical" | "warning" | "info"

export interface SeoIssue {
  page_id: string
  severity: IssueType
  title: string
  description: string
  page_url: string
  element: string | null
  line_number: number | null
  recommendation: string
} 

export interface CompleteAnalysisResult {
  analysis: AnalysisResults
  pages: PageAnalysisData[]
  issues: SeoIssue[]
  summary: AnalysisSummary
}

// ============================================================================
// Extended types for detailed page view (backend will populate these)
// ============================================================================

export interface HeadingElement {
  tag: 'h1' | 'h2' | 'h3' | 'h4' | 'h5' | 'h6'
  text: string
}

export interface ImageElement {
  src: string
  alt: string | null
}

export interface LinkElement {
  href: string
  text: string
  is_internal: boolean
  status_code: number | null
}

// Extended page data with detailed elements
export interface PageDetailData extends PageAnalysisData {
  headings?: HeadingElement[]
  images?: ImageElement[]
  detailed_links?: LinkElement[]
}
