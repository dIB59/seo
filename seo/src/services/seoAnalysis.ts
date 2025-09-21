// services/seoAnalysis.ts
import {
    AnalysisResult,
    AnalysisSettings,
    PageAnalysis
} from '../types/seo';
import {
    AnalysisListResponse,
    AnalysisProgressEvent,
    AnalysisCompleteEvent,
    AnalysisErrorEvent
} from '../types/api';

/**
 * Start a new SEO analysis
 */
export const startAnalysis = async (url: string, settings?: Partial<AnalysisSettings>): Promise<string> => {
    // Simulate API delay
    await new Promise(resolve => setTimeout(resolve, 500));

    // Return dummy analysis ID
    const analysisId = `analysis_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    console.log('Starting analysis for:', url, 'with settings:', settings);

    return analysisId;
};

/**
 * Get analysis progress
 */
export const getAnalysisProgress = async (analysisId: string): Promise<AnalysisResult> => {
    // Simulate API delay
    await new Promise(resolve => setTimeout(resolve, 300));

    // Simulate progressive analysis
    const startTime = new Date(Date.now() - Math.random() * 300000).toISOString(); // Started 0-5 minutes ago
    const progress = Math.min(100, Math.random() * 100);
    const totalPages = 25 + Math.floor(Math.random() * 75); // 25-100 pages
    const analyzedPages = Math.floor((progress / 100) * totalPages);

    const dummyResult: AnalysisResult = {
        id: analysisId,
        url: 'https://example.com',
        status: progress >= 100 ? 'completed' : 'analyzing',
        progress,
        total_pages: totalPages,
        analyzed_pages: analyzedPages,
        started_at: startTime,
        completed_at: progress >= 100 ? new Date().toISOString() : undefined,
        pages: [],
        issues: {
            critical: Math.floor(Math.random() * 10),
            warnings: Math.floor(Math.random() * 20),
            suggestions: Math.floor(Math.random() * 30),
        },
        summary: {
            avg_load_time: 1.2 + Math.random() * 2,
            total_words: Math.floor(Math.random() * 50000) + 10000,
            pages_with_issues: Math.floor(Math.random() * totalPages),
            seo_score: Math.floor(Math.random() * 40) + 60, // 60-100
            mobile_friendly_pages: Math.floor(Math.random() * totalPages),
            pages_with_meta_description: Math.floor(Math.random() * totalPages),
            pages_with_title_issues: Math.floor(Math.random() * 5),
            duplicate_titles: Math.floor(Math.random() * 3),
            duplicate_meta_descriptions: Math.floor(Math.random() * 3),
        },
        sitemap_found: Math.random() > 0.3,
        robots_txt_found: Math.random() > 0.2,
        ssl_certificate: Math.random() > 0.1,
    };

    console.log('Progress for analysis:', analysisId, dummyResult);
    return dummyResult;
};

/**
 * Get completed analysis results
 */
export const getAnalysis = async (analysisId: string): Promise<AnalysisResult> => {
    await new Promise(resolve => setTimeout(resolve, 200));

    const dummyPages: PageAnalysis[] = Array.from({ length: 25 }, (_, i) => {
        const pageTypes = ['/', '/about', '/contact', '/blog', '/services', '/products', '/privacy', '/terms'];
        const basePath = pageTypes[i % pageTypes.length] || '/';
        const pageUrl = i === 0 ? url : `${url.replace(/\/$/, '')}${basePath}${i > 7 ? `/${i}` : ''}`;

        const statusCodes = [200, 200, 200, 200, 200, 301, 302, 404, 500];
        const statusCode = statusCodes[Math.floor(Math.random() * statusCodes.length)];

        const hasIssues = Math.random() > 0.6;
        const issues: any[] = hasIssues ? [
            {
                id: `issue_${i}_1`,
                type: Math.random() > 0.7 ? 'critical' : Math.random() > 0.5 ? 'warning' : 'suggestion',
                title: 'Missing meta description',
                description: 'This page is missing a meta description which is important for SEO.',
                page_url: pageUrl,
                element: '<head>',
                recommendation: 'Add a unique meta description between 150-160 characters.',
            },
            ...(Math.random() > 0.7 ? [{
                id: `issue_${i}_2`,
                type: 'warning' as const,
                title: 'Multiple H1 tags',
                description: 'This page has multiple H1 tags which can confuse search engines.',
                page_url: pageUrl,
                element: '<h1>',
                recommendation: 'Use only one H1 tag per page.',
            }] : [])
        ] : [];

        return {
            url: pageUrl,
            title: i === 0
                ? `${new URL(url).hostname} - Home Page`
                : `${basePath.replace('/', '').replace(/^\w/, c => c.toUpperCase())} Page - ${new URL(url).hostname}`,
            meta_description: Math.random() > 0.3
                ? `This is the meta description for the ${basePath} page. It provides a brief summary of the page content.`
                : undefined,
            meta_keywords: Math.random() > 0.7 ? 'seo, analysis, page, content, website' : undefined,
            canonical_url: pageUrl,
            h1_count: Math.random() > 0.8 ? 0 : Math.random() > 0.9 ? 2 : 1,
            h2_count: 2 + Math.floor(Math.random() * 4),
            h3_count: 3 + Math.floor(Math.random() * 6),
            image_count: Math.floor(Math.random() * 15),
            images_without_alt: Math.floor(Math.random() * 3),
            internal_links: 5 + Math.floor(Math.random() * 20),
            external_links: Math.floor(Math.random() * 8),
            word_count: 200 + Math.floor(Math.random() * 2000),
            load_time: 0.5 + Math.random() * 4,
            status_code: statusCode,
            content_size: Math.floor(Math.random() * 150000) + 20000,
            mobile_friendly: Math.random() > 0.2,
            has_structured_data: Math.random() > 0.4,
            lighthouse_score: statusCode === 200 ? {
                performance: Math.floor(Math.random() * 40) + 60,
                accessibility: Math.floor(Math.random() * 30) + 70,
                best_practices: Math.floor(Math.random() * 30) + 70,
                seo: Math.floor(Math.random() * 20) + 80,
            } : undefined,
            issues,
            created_at: new Date().toISOString(),
        };
    });

    const dummyResult: AnalysisResult = {
        id: analysisId,
        url: 'https://example.com',
        status: 'completed',
        progress: 100,
        total_pages: 25,
        analyzed_pages: 25,
        started_at: new Date(Date.now() - 300000).toISOString(), // 5 minutes ago
        completed_at: new Date().toISOString(),
        pages: dummyPages,
        issues: {
            critical: 3,
            warnings: 12,
            suggestions: 18,
        },
        summary: {
            avg_load_time: 2.1,
            total_words: 35000,
            pages_with_issues: 15,
            seo_score: 78,
            mobile_friendly_pages: 22,
            pages_with_meta_description: 20,
            pages_with_title_issues: 2,
            duplicate_titles: 1,
            duplicate_meta_descriptions: 1,
        },
        sitemap_found: true,
        robots_txt_found: true,
        ssl_certificate: true,
    };

    return dummyResult;
};

/**
 * Get list of all analyses
 */
export const getAnalysisList = async (page = 1, perPage = 10): Promise<AnalysisListResponse> => {
    await new Promise(resolve => setTimeout(resolve, 400));

    const dummyAnalyses: AnalysisResult[] = Array.from({ length: 5 }, (_, i) => {
        const isCompleted = Math.random() > 0.3;
        const startTime = new Date(Date.now() - Math.random() * 86400000 * 7).toISOString(); // Within last week

        return {
            id: `analysis_${Date.now() - i * 1000}_${Math.random().toString(36).substr(2, 9)}`,
            url: `https://example-${i + 1}.com`,
            status: isCompleted ? 'completed' : (Math.random() > 0.7 ? 'analyzing' : 'paused'),
            progress: isCompleted ? 100 : Math.floor(Math.random() * 90) + 10,
            total_pages: 20 + Math.floor(Math.random() * 80),
            analyzed_pages: isCompleted ? 20 + Math.floor(Math.random() * 80) : Math.floor(Math.random() * 15) + 5,
            started_at: startTime,
            completed_at: isCompleted ? new Date(new Date(startTime).getTime() + Math.random() * 3600000).toISOString() : undefined,
            pages: [],
            issues: {
                critical: Math.floor(Math.random() * 8),
                warnings: Math.floor(Math.random() * 25),
                suggestions: Math.floor(Math.random() * 35),
            },
            summary: {
                avg_load_time: 1 + Math.random() * 3,
                total_words: Math.floor(Math.random() * 80000) + 20000,
                pages_with_issues: Math.floor(Math.random() * 20) + 5,
                seo_score: Math.floor(Math.random() * 50) + 50,
                mobile_friendly_pages: Math.floor(Math.random() * 20) + 15,
                pages_with_meta_description: Math.floor(Math.random() * 20) + 10,
                pages_with_title_issues: Math.floor(Math.random() * 5),
                duplicate_titles: Math.floor(Math.random() * 3),
                duplicate_meta_descriptions: Math.floor(Math.random() * 4),
            },
            sitemap_found: Math.random() > 0.2,
            robots_txt_found: Math.random() > 0.3,
            ssl_certificate: Math.random() > 0.1,
        };
    });

    return {
        analyses: dummyAnalyses,
        total: 15,
        page,
        per_page: perPage,
    };
};

/**
 * Delete an analysis
 */
export const deleteAnalysis = async (analysisId: string): Promise<void> => {
    await new Promise(resolve => setTimeout(resolve, 200));
    console.log('Deleted analysis:', analysisId);
};

/**
 * Pause an ongoing analysis
 */
export const pauseAnalysis = async (analysisId: string): Promise<void> => {
    await new Promise(resolve => setTimeout(resolve, 100));
    console.log('Paused analysis:', analysisId);
};

/**
 * Resume a paused analysis
 */
export const resumeAnalysis = async (analysisId: string): Promise<void> => {
    await new Promise(resolve => setTimeout(resolve, 100));
    console.log('Resumed analysis:', analysisId);
};

/**
 * Export analysis report
 */
export const exportReport = async (analysisId: string, format: 'pdf' | 'csv' | 'json'): Promise<string> => {
    await new Promise(resolve => setTimeout(resolve, 1000)); // Simulate export time

    const fileName = `seo-report-${analysisId.split('_')[1]}.${format}`;
    const filePath = `/Users/username/Downloads/${fileName}`;

    console.log(`Exported ${format.toUpperCase()} report:`, filePath);

    // Simulate file creation success
    return filePath;
};

/**
 * Listen for analysis progress updates
 */
export const onProgressUpdate = (callback: (event: AnalysisProgressEvent) => void) => {
    // Simulate real-time progress updates for demo
    console.log('Setting up progress listener');

    // Return a dummy unlisten function
    return Promise.resolve(() => {
        console.log('Unlistened from progress updates');
    });
};

/**
 * Listen for analysis completion
 */
export const onAnalysisComplete = (callback: (event: AnalysisCompleteEvent) => void) => {
    console.log('Setting up completion listener');

    // Return a dummy unlisten function
    return Promise.resolve(() => {
        console.log('Unlistened from completion events');
    });
};

/**
 * Listen for analysis errors
 */
export const onAnalysisError = (callback: (event: AnalysisErrorEvent) => void) => {
    console.log('Setting up error listener');

    // Return a dummy unlisten function
    return Promise.resolve(() => {
        console.log('Unlistened from error events');
    });
};

/**
 * Utility functions
 */
export const seoUtils = {
    /**
     * Validate URL format
     */
    isValidUrl: (url: string): boolean => {
        try {
            new URL(url);
            return true;
        } catch {
            return false;
        }
    },

    /**
     * Format URL for display
     */
    formatUrl: (url: string): string => {
        try {
            const urlObj = new URL(url);
            return `${urlObj.protocol}//${urlObj.host}${urlObj.pathname}`;
        } catch {
            return url;
        }
    },

    /**
     * Calculate SEO score color
     */
    getScoreColor: (score: number): string => {
        if (score >= 80) return 'text-green-600';
        if (score >= 60) return 'text-yellow-600';
        return 'text-red-600';
    },

    /**
     * Format analysis duration
     */
    formatDuration: (startTime: string, endTime?: string): string => {
        const start = new Date(startTime);
        const end = endTime ? new Date(endTime) : new Date();
        const diffMs = end.getTime() - start.getTime();
        const diffMinutes = Math.floor(diffMs / 60000);
        const diffSeconds = Math.floor((diffMs % 60000) / 1000);

        if (diffMinutes > 0) {
            return `${diffMinutes}m ${diffSeconds}s`;
        }
        return `${diffSeconds}s`;
    },

    /**
     * Format file size
     */
    formatFileSize: (bytes: number): string => {
        if (bytes === 0) return '0 Bytes';
        const k = 1024;
        const sizes = ['Bytes', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }
};