// hooks/useSeoAnalysis.tr
import { useState, useEffect, useCallback } from 'react';
import { AnalysisProgress, AnalysisSettings } from '../types/seo';
import {
	startAnalysis,
	getAnalysisList,
	getAnalysisProgress,
	pauseAnalysis as pauseAnalysisService,
	resumeAnalysis as resumeAnalysisService,
	deleteAnalysis as deleteAnalysisService,
	exportReport as exportReportService,
	onProgressUpdate,
	onAnalysisComplete,
	onAnalysisError
} from '../services/mockSeoAnalysis';
import { AnalysisProgressEvent, AnalysisCompleteEvent, AnalysisErrorEvent } from '../types/api';

interface UseSeoAnalysisState {
	currentAnalysis: AnalysisProgress | null;
	recentAnalyses: AnalysisProgress[];
	isAnalyzing: boolean;
	error: string | null;
	isLoading: boolean;
}

interface UseSeoAnalysisActions {
	startAnalysis: (url: string, settings?: Partial<AnalysisSettings>) => Promise<void>;
	pauseAnalysis: (analysisId: number) => Promise<void>;
	resumeAnalysis: (analysisId: number) => Promise<void>;
	deleteAnalysis: (analysisId: number) => Promise<void>;
	exportReport: (analysisId: string, format: 'pdf' | 'csv' | 'json') => Promise<string>;
	clearError: () => void;
	refreshAnalyses: () => Promise<void>;
}

export function useSeoAnalysis(): UseSeoAnalysisState & UseSeoAnalysisActions {
	const [state, setState] = useState<UseSeoAnalysisState>({
		currentAnalysis: null,
		recentAnalyses: [],
		isAnalyzing: false,
		error: null,
		isLoading: false,
	});

	// Load recent analyses on mount
	useEffect(() => {
		loadRecentAnalyses();
	}, []);

	// Set up event listeners for real-time updates
	useEffect(() => {
		const unlistenProgress = onProgressUpdate(handleProgressUpdate);
		const unlistenComplete = onAnalysisComplete(handleAnalysisComplete);
		const unlistenError = onAnalysisError(handleAnalysisError);

		return () => {
			unlistenProgress.then(fn => fn());
			unlistenComplete.then(fn => fn());
			unlistenError.then(fn => fn());
		};
	}, []);

	const loadRecentAnalyses = async () => {
		try {
			setState(prev => ({ ...prev, isLoading: true, error: null }));
			const response = await getAnalysisList(1, 10);
			setState(prev => ({
				...prev,
				recentAnalyses: response.analyses,
				isLoading: false
			}));
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'Failed to load analyses';
			setState(prev => ({
				...prev,
				error: errorMessage,
				isLoading: false
			}));
		}
	};


	const handleProgressUpdate = (event: AnalysisProgressEvent) => {
		setState(prev => {
			// Narrow prev.currentAnalysis so TS knows it's an AnalysisResult
			if (prev.currentAnalysis && prev.currentAnalysis.job_id === parseInt(event.analysis_id)) {
				const curr = prev.currentAnalysis; // now narrowed to AnalysisResult
				return {
					...prev,
					currentAnalysis: {
						...curr,
						// keep all existing AnalysisResult fields, only update progress-related ones
						progress: event.progress,
						analyzed_pages: event.analyzed_pages,
						total_pages: event.total_pages,
					}
				};
			}
			return prev;
		});
	};


	const handleAnalysisComplete = (event: AnalysisCompleteEvent) => {
		setState(prev => ({
			...prev,
			currentAnalysis: event.result,
			isAnalyzing: false,
			recentAnalyses: [event.result, ...prev.recentAnalyses.slice(0, 9)]
		}));
	};

	const handleAnalysisError = (event: AnalysisErrorEvent) => {
		setState(prev => ({
			...prev,
			error: `Analysis failed: ${event.error}`,
			isAnalyzing: false,
			currentAnalysis: null,
		}));
	};

	const startAnalysisHandler = useCallback(async (url: string, settings?: Partial<AnalysisSettings>) => {
		try {
			setState(prev => ({
				...prev,
				error: null,
				isAnalyzing: true,
				currentAnalysis: null
			}));

			const analysisId = await startAnalysis(url, settings);

			// Get initial analysis state
			const initialAnalysis = await getAnalysisProgress(analysisId);

			setState(prev => ({
				...prev,
				currentAnalysis: initialAnalysis,

			}));

		} catch (error) {
			console.log(error);
			const errorMessage = error instanceof Error ? error.message : 'Failed to start analysis';
			setState(prev => ({
				...prev,
				error: errorMessage,
				isAnalyzing: false
			}));
		}
	}, []);

	const pauseAnalysisHandler = useCallback(async (analysisId: number) => {
		try {
			await pauseAnalysisService(String(analysisId));
			setState(prev => {
				if (prev.currentAnalysis?.job_id === analysisId) {
					return {
						...prev,
						currentAnalysis: {
							...prev.currentAnalysis,
							status: 'paused'
						}
					};
				}
				return prev;
			});
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'Failed to pause analysis';
			setState(prev => ({ ...prev, error: errorMessage }));
		}
	}, []);

	const resumeAnalysisHandler = useCallback(async (analysisId: number) => {
		try {
			await resumeAnalysisService(String(analysisId));
			setState(prev => {
				if (prev.currentAnalysis?.job_id === analysisId) {
					return {
						...prev,
						currentAnalysis: {
							...prev.currentAnalysis,
							status: 'analyzing'
						},
						isAnalyzing: true
					};
				}
				return prev;
			});
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'Failed to resume analysis';
			setState(prev => ({ ...prev, error: errorMessage }));
		}
	}, []);

	const deleteAnalysisHandler = useCallback(async (analysisId: number) => {
		try {
			await deleteAnalysisService(analysisId);
			setState(prev => ({
				...prev,
				recentAnalyses: prev.recentAnalyses.filter(analysis => analysis.job_id !== analysisId),
				currentAnalysis: prev.currentAnalysis?.job_id === analysisId ? null : prev.currentAnalysis
			}));
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'Failed to delete analysis';
			setState(prev => ({ ...prev, error: errorMessage }));
		}
	}, []);

	const exportReportHandler = useCallback(async (analysisId: string, format: 'pdf' | 'csv' | 'json'): Promise<string> => {
		try {
			const filePath = await exportReportService(analysisId, format);
			return filePath;
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'Failed to export report';
			setState(prev => ({ ...prev, error: errorMessage }));
			throw error;
		}
	}, []);

	const clearError = useCallback(() => {
		setState(prev => ({ ...prev, error: null }));
	}, []);

	const refreshAnalyses = useCallback(async () => {
		await loadRecentAnalyses();
	}, []);

	return {
		...state,
		startAnalysis: startAnalysisHandler,
		pauseAnalysis: pauseAnalysisHandler,
		resumeAnalysis: resumeAnalysisHandler,
		deleteAnalysis: deleteAnalysisHandler,
		exportReport: exportReportHandler,
		clearError,
		refreshAnalyses,
	};
}
