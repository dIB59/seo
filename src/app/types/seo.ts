// types/seo.ts
export interface SeoIssue {
    id: string;
    type: 'critical' | 'warning' | 'suggestion';
    title: string;
    description: string;
    page_url: string;
    element?: string;
    line_number?: number;
    recommendation: string;
}

export interface PageAnalysis {
    url: string;
    title?: string;
    meta_description?: string;
    meta_keywords?: string;
    canonical_url?: string;
    h1_count: number;
    h2_count: number;
    h3_count: number;
    image_count: number;
    images_without_alt: number;
    internal_links: number;
        external_links: number;
    word_count: number;
    load_time: number;
    status_code: number;
    content_size: number;
    mobile_friendly: boolean;
    has_structured_data: boolean;
    lighthouse_score?: {
        performance: number;
        accessibility: number;
        best_practices: number;
        seo: number;
    };
    issues: SeoIssue[];
    created_at: string;
}

export interface AnalysisResult {
    id: string;
    url: string;
    status: 'analyzing' | 'completed' | 'error' | 'paused';
    progress: number;
    total_pages: number;
    analyzed_pages: number;
    started_at: string;
    completed_at?: string;
    pages: PageAnalysis[];
    issues: {
        critical: number;
        warnings: number;
        suggestions: number;
    };
    summary: {
        avg_load_time: number;
        total_words: number;
        pages_with_issues: number;
        seo_score: number; // 0-100
        mobile_friendly_pages: number;
        pages_with_meta_description: number;
        pages_with_title_issues: number;
        duplicate_titles: number;
        duplicate_meta_descriptions: number;
    };
    sitemap_found: boolean;
    robots_txt_found: boolean;
    ssl_certificate: boolean;
}

export interface AnalysisError {
    message: string;
    code: string;
    details?: Record<string, any>;
}

export interface AnalysisSettings {
    max_pages: number;
    include_external_links: boolean;
    check_images: boolean;
    mobile_analysis: boolean;
    lighthouse_analysis: boolean;
    delay_between_requests: number; // milliseconds
}

export const defaultSettings: AnalysisSettings = {
  max_pages: 10,
  include_external_links: false,
  check_images: true,
  mobile_analysis: false,
  lighthouse_analysis: false,
  delay_between_requests: 500,
};