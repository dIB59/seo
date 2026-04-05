export interface PromptBlock {
  id: string;
  type: "text" | "variable";
  content: string;
}

export interface LighthouseAuditResult {
  passed: boolean;
  value: string | null;
  score: number;
  description?: string;
}

export interface LighthouseSeoAudits {
  document_title: LighthouseAuditResult;
  meta_description: LighthouseAuditResult;
  viewport: LighthouseAuditResult;
  canonical: LighthouseAuditResult;
  hreflang: LighthouseAuditResult;
  crawlable_anchors: LighthouseAuditResult;
  link_text: LighthouseAuditResult;
  image_alt: LighthouseAuditResult;
  http_status_code: LighthouseAuditResult;
  is_crawlable: LighthouseAuditResult;
}

export interface LighthousePerformanceMetrics {
  first_contentful_paint: number | null;
  largest_contentful_paint: number | null;
  speed_index: number | null;
  time_to_interactive: number | null;
  total_blocking_time: number | null;
  cumulative_layout_shift: number | null;
}
