import {
    PageAnalysisData
} from "./bindings";

// Re-export common types from bindings for convenience
export * from "./bindings";

// Detailed Lighthouse audit result for a single check
export interface LighthouseAuditResult {
    passed: boolean;
    value: string | null;
    score: number;
    description?: string;
}

// SEO-specific Lighthouse audits
export interface LighthouseSeoAudits {
    document_title: LighthouseAuditResult;
    meta_description: LighthouseAuditResult;
    viewport: LighthouseAuditResult;
    canonical: LighthouseAuditResult;
    hreflang: LighthouseAuditResult;
    robots_txt: LighthouseAuditResult;
    crawlable_anchors: LighthouseAuditResult;
    link_text: LighthouseAuditResult;
    image_alt: LighthouseAuditResult;
    http_status_code: LighthouseAuditResult;
    is_crawlable: LighthouseAuditResult;
}

// Core Web Vitals and performance metrics
export interface LighthousePerformanceMetrics {
    first_contentful_paint: number | null;
    largest_contentful_paint: number | null;
    speed_index: number | null;
    time_to_interactive: number | null;
    total_blocking_time: number | null;
    cumulative_layout_shift: number | null;
}

// Map the severity to a simpler type if needed
export type IssueType = "critical" | "warning" | "info";

// LinkElement alias used in some components
export interface LinkElement {
    href: string;
    text: string;
    is_external: boolean;
    status_code: number | null;
}

// Extended page data with detailed elements (used in PageDetailView)
export interface PageDetailData extends Omit<PageAnalysisData, 'lighthouse_seo_audits' | 'lighthouse_performance_metrics'> {
    lighthouse_seo_audits?: LighthouseSeoAudits | null;
    lighthouse_performance_metrics?: LighthousePerformanceMetrics | null;
}
