// services/seoAnalysis.ts
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import {
    AnalysisResult,
    AnalysisSettings
} from '../types/seo';
import {
    TauriResponse,
    AnalysisListResponse,
    AnalysisProgressEvent,
    AnalysisCompleteEvent,
    AnalysisErrorEvent
} from '../types/api';

/**
 * Start a new SEO analysis
 */
export const startAnalysis = async (url: string, settings?: Partial<AnalysisSettings>): Promise<string> => {
    const response = await invoke<TauriResponse<string>>('start_seo_analysis', { url, settings });

    if (!response.success || !response.data) {
        throw new Error(response.error || 'Failed to start analysis');
    }

    return response.data;
};

/**
 * Get analysis progress
 */
export const getAnalysisProgress = async (analysisId: string): Promise<AnalysisResult> => {
    const response = await invoke<TauriResponse<AnalysisResult>>('get_analysis_progress', {
        analysis_id: analysisId
    });

    if (!response.success || !response.data) {
        throw new Error(response.error || 'Failed to get analysis progress');
    }

    return response.data;
};

/**
 * Get completed analysis results
 */
export const getAnalysis = async (analysisId: string): Promise<AnalysisResult> => {
    const response = await invoke<TauriResponse<AnalysisResult>>('get_analysis', {
        analysis_id: analysisId
    });

    if (!response.success || !response.data) {
        throw new Error(response.error || 'Failed to get analysis');
    }

    return response.data;
};

/**
 * Get list of all analyses
 */
export const getAnalysisList = async (page = 1, perPage = 10): Promise<AnalysisListResponse> => {
    const response = await invoke<TauriResponse<AnalysisListResponse>>('get_analysis_list', {
        page,
        per_page: perPage
    });

    if (!response.success || !response.data) {
        throw new Error(response.error || 'Failed to get analysis list');
    }

    return response.data;
};

/**
 * Delete an analysis
 */
export const deleteAnalysis = async (analysisId: string): Promise<void> => {
    const response = await invoke<TauriResponse<void>>('delete_analysis', {
        analysis_id: analysisId
    });

    if (!response.success) {
        throw new Error(response.error || 'Failed to delete analysis');
    }
};

/**
 * Pause an ongoing analysis
 */
export const pauseAnalysis = async (analysisId: string): Promise<void> => {
    const response = await invoke<TauriResponse<void>>('pause_analysis', {
        analysis_id: analysisId
    });

    if (!response.success) {
        throw new Error(response.error || 'Failed to pause analysis');
    }
};

/**
 * Resume a paused analysis
 */
export const resumeAnalysis = async (analysisId: string): Promise<void> => {
    const response = await invoke<TauriResponse<void>>('resume_analysis', {
        analysis_id: analysisId
    });

    if (!response.success) {
        throw new Error(response.error || 'Failed to resume analysis');
    }
};

/**
 * Export analysis report
 */
export const exportReport = async (analysisId: string, format: 'pdf' | 'csv' | 'json'): Promise<string> => {
    const response = await invoke<TauriResponse<string>>('export_report', {
        analysis_id: analysisId,
        format
    });

    if (!response.success || !response.data) {
        throw new Error(response.error || 'Failed to export report');
    }

    return response.data; // Returns file path
};

/**
 * Listen for analysis progress updates
 */
export const onProgressUpdate = (callback: (event: AnalysisProgressEvent) => void) => {
    return listen<AnalysisProgressEvent>('analysis-progress', (event) => {
        callback(event.payload);
    });
};

/**
 * Listen for analysis completion
 */
export const onAnalysisComplete = (callback: (event: AnalysisCompleteEvent) => void) => {
    return listen<AnalysisCompleteEvent>('analysis-complete', (event) => {
        callback(event.payload);
    });
};

/**
 * Listen for analysis errors
 */
export const onAnalysisError = (callback: (event: AnalysisErrorEvent) => void) => {
    return listen<AnalysisErrorEvent>('analysis-error', (event) => {
        callback(event.payload);
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