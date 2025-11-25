import { invoke } from '@tauri-apps/api/core';

// services/seoAnalysis.ts
import {
	AnalysisJobResponse,
	AnalysisProgress,
	AnalysisResult,
	AnalysisSettings,
	defaultSettings,
	PageAnalysis
} from '../types/seo';
import {
	AnalysisListResponse,
	AnalysisProgressEvent,
	AnalysisCompleteEvent,
	AnalysisErrorEvent,
	TauriResponse
} from '../types/api';

/**
 * Start a new SEO analysis
 */
export const startAnalysis = async (
	url: string,
	settings: Partial<AnalysisSettings>
): Promise<number> => {
	const merged: AnalysisSettings = { ...defaultSettings, ...settings };
	console.log(merged);
	const analysisId = await invoke<AnalysisJobResponse>('start_analysis', { url, settings: merged });
	console.log(analysisId);
	return analysisId.job_id;
};

/**
 * Get progress for a specific analysis job
 */
export const getAnalysisProgress = async (jobId: number): Promise<AnalysisProgress> => {
	const progress = await invoke<TauriResponse<AnalysisProgress>>('get_analysis_progress', { jobId });
	console.log('Progress for job:', jobId, progress);

	if ('error' in progress) {
		throw new Error(progress.error);
	}
	return progress.data;

};

/**
 * Get completed analysis results
 */
export const getAnalysis = async (analysisId: string): Promise<AnalysisResult> => {
	await new Promise(resolve => setTimeout(resolve, 200));

	const dummyPages: PageAnalysis[] = Array.from({ length: 5 }, (_, i) => ({
		url: `https://example.com/page-${i + 1}`,
		title: `Page ${i + 1} Title - Example Site`,
		meta_description: `This is the meta description for page ${i + 1}. It describes the content of the page.`,
		meta_keywords: 'seo, analysis, page, content',
		canonical_url: `https://example.com/page-${i + 1}`,
		h1_count: 1,
		h2_count: 2 + Math.floor(Math.random() * 4),
		h3_count: 3 + Math.floor(Math.random() * 6),
		image_count: 5 + Math.floor(Math.random() * 10),
		images_without_alt: Math.floor(Math.random() * 3),
		internal_links: 10 + Math.floor(Math.random() * 20),
		external_links: 2 + Math.floor(Math.random() * 8),
		word_count: 500 + Math.floor(Math.random() * 1500),
		load_time: 1 + Math.random() * 3,
		status_code: 200,
		content_size: Math.floor(Math.random() * 100000) + 50000,
		mobile_friendly: Math.random() > 0.2,
		has_structured_data: Math.random() > 0.4,
		lighthouse_score: {
			performance: Math.floor(Math.random() * 40) + 60,
			accessibility: Math.floor(Math.random() * 30) + 70,
			best_practices: Math.floor(Math.random() * 30) + 70,
			seo: Math.floor(Math.random() * 20) + 80,
		},
		issues: [
			{
				id: `issue_${i}_1`,
				type: 'warning' as const,
				title: 'Missing alt text on image',
				description: 'Some images are missing alternative text which is important for accessibility.',
				page_url: `https://example.com/page-${i + 1}`,
				element: '<img src="/image.jpg">',
				recommendation: 'Add descriptive alt text to all images.',
			}
		],
		created_at: new Date().toISOString(),
	}));

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
export const deleteAnalysis = async (analysisId: number): Promise<void> => {
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
