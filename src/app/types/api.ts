// types/api.ts
import { AnalysisResult, AnalysisSettings } from './seo';

// Tauri command parameters - using Record<string, unknown> for compatibility
export interface StartAnalysisParams extends Record<string, unknown> {
    url: string;
    settings?: Partial<AnalysisSettings>;
}

export interface GetProgressParams extends Record<string, unknown> {
    analysis_id: string;
}

export interface GetAnalysisParams extends Record<string, unknown> {
    analysis_id: string;
}

export interface DeleteAnalysisParams extends Record<string, unknown> {
    analysis_id: string;
}

export interface ExportReportParams extends Record<string, unknown> {
    analysis_id: string;
    format: 'pdf' | 'csv' | 'json';
}

export interface PauseAnalysisParams extends Record<string, unknown> {
    analysis_id: string;
}

export interface ResumeAnalysisParams extends Record<string, unknown> {
    analysis_id: string;
}

// Tauri command responses
export interface TauriResponse<T> {
    success: boolean;
    data?: T;
    error?: string;
}

export interface AnalysisListResponse {
    analyses: AnalysisResult[];
    total: number;
    page: number;
    per_page: number;
}

// Event types for real-time updates
export interface AnalysisProgressEvent {
    analysis_id: string;
    progress: number;
    analyzed_pages: number;
    total_pages: number;
    current_page?: string;
}

export interface AnalysisCompleteEvent {
    analysis_id: string;
    result: AnalysisResult;
}

export interface AnalysisErrorEvent {
    analysis_id: string;
    error: string;
}