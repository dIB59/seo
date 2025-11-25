// Types matching the Tauri backend

export interface AnalysisJobResponse {
  job_id: number
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
  job_id: number
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
}

export type IssueType = "Critical" | "Warning" | "Suggestion"

export interface SeoIssue {
  page_id: string
  issue_type: IssueType
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
