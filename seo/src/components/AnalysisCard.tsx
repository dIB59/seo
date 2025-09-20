// components/AnalysisCard.tsx
import React from 'react';
import { AnalysisResult } from '../types/seo';
import { seoUtils } from '../services/seoAnalysis';

interface AnalysisCardProps {
    analysis: AnalysisResult;
    onView: (analysis: AnalysisResult) => void;
    onDelete: (analysisId: string) => void;
    onPause?: (analysisId: string) => void;
    onResume?: (analysisId: string) => void;
    onExport?: (analysisId: string, format: 'pdf' | 'csv' | 'json') => void;
}

export const AnalysisCard: React.FC<AnalysisCardProps> = ({
    analysis,
    6onView,
    onDelete,
    onPause,
    onResume,
    onExport,
}) => {
    const getStatusBadge = () => {
        const statusConfig = {
            analyzing: { color: 'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-400', text: 'Analyzing' },
            completed: { color: 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400', text: 'Completed' },
            error: { color: 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400', text: 'Error' },
            paused: { color: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-400', text: 'Paused' },
        };

        const config = statusConfig[analysis.status];
        return (
            <span className={`px-2 py-1 rounded-full text-xs font-medium ${config.color}`}>
                {config.text}
            </span>
        );
    };

    const getScoreColor = (score: number) => {
        if (score >= 80) return 'text-green-600 dark:text-green-400';
        if (score >= 60) return 'text-yellow-600 dark:text-yellow-400';
        return 'text-red-600 dark:text-red-400';
    };

    const formatDate = (dateString: string) => {
        return new Date(dateString).toLocaleDateString('en-US', {
            month: 'short',
            day: 'numeric',
            year: 'numeric',
            hour: '2-digit',
            minute: '2-digit',
        });
    };

    return (
        <div className="bg-white dark:bg-gray-800 rounded-xl shadow-lg hover:shadow-xl transition-all duration-300 border border-gray-200 dark:border-gray-700">
            <div className="p-6">
                {/* Header */}
                <div className="flex items-start justify-between mb-4">
                    <div className="flex-1 min-w-0">
                        <h3 className="text-lg font-semibold text-gray-900 dark:text-white truncate">
                            {seoUtils.formatUrl(analysis.url)}
                        </h3>
                        <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
                            Started: {formatDate(analysis.started_at)}
                            {analysis.completed_at && (
                                <span className="ml-2">
                                    • Duration: {seoUtils.formatDuration(analysis.started_at, analysis.completed_at)}
                                </span>
                            )}
                        </p>
                    </div>
                    <div className="flex items-center gap-2 ml-4">
                        {getStatusBadge()}
                    </div>
                </div>

                {/* Progress Bar (for analyzing status) */}
                {analysis.status === 'analyzing' && (
                    <div className="mb-4">
                        <div className="flex justify-between text-sm text-gray-600 dark:text-gray-400 mb-2">
                            <span>Progress: {Math.round(analysis.progress)}%</span>
                            <span>{analysis.analyzed_pages} / {analysis.total_pages || '?'} pages</span>
                        </div>
                        <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                            <div
                                className="bg-blue-500 h-2 rounded-full transition-all duration-500"
                                style={{ width: `${analysis.progress}%` }}
                            />
                        </div>
                    </div>
                )}

                {/* SEO Score (for completed analyses) */}
                {analysis.status === 'completed' && (
                    <div className="mb-4">
                        <div className="flex items-center justify-between">
                            <span className="text-sm font-medium text-gray-600 dark:text-gray-400">SEO Score</span>
                            <span className={`text-2xl font-bold ${getScoreColor(analysis.summary.seo_score)}`}>
                                {analysis.summary.seo_score}/100
                            </span>
                        </div>
                        <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2 mt-2">
                            <div
                                className={`h-2 rounded-full transition-all duration-500 ${analysis.summary.seo_score >= 80 ? 'bg-green-500' :
                                    analysis.summary.seo_score >= 60 ? 'bg-yellow-500' : 'bg-red-500'
                                    }`}
                                style={{ width: `${analysis.summary.seo_score}%` }}
                            />
                        </div>
                    </div>
                )}

                {/* Issue Summary (for completed analyses) */}
                {analysis.status === 'completed' && (
                    <div className="grid grid-cols-3 gap-4 mb-4">
                        <div className="text-center">
                            <div className="flex items-center justify-center gap-1">
                                <div className="w-2 h-2 bg-red-500 rounded-full" />
                                <span className="text-lg font-semibold text-gray-900 dark:text-white">
                                    {analysis.issues.critical}
                                </span>
                            </div>
                            <p className="text-xs text-gray-500 dark:text-gray-400">Critical</p>
                        </div>
                        <div className="text-center">
                            <div className="flex items-center justify-center gap-1">
                                <div className="w-2 h-2 bg-yellow-500 rounded-full" />
                                <span className="text-lg font-semibold text-gray-900 dark:text-white">
                                    {analysis.issues.warnings}
                                </span>
                            </div>
                            <p className="text-xs text-gray-500 dark:text-gray-400">Warnings</p>
                        </div>
                        <div className="text-center">
                            <div className="flex items-center justify-center gap-1">
                                <div className="w-2 h-2 bg-blue-500 rounded-full" />
                                <span className="text-lg font-semibold text-gray-900 dark:text-white">
                                    {analysis.issues.suggestions}
                                </span>
                            </div>
                            <p className="text-xs text-gray-500 dark:text-gray-400">Suggestions</p>
                        </div>
                    </div>
                )}

                {/* Quick Stats (for completed analyses) */}
                {analysis.status === 'completed' && (
                    <div className="grid grid-cols-2 gap-4 text-sm text-gray-600 dark:text-gray-400 mb-4">
                        <div>
                            <span className="font-medium">{analysis.total_pages}</span> pages analyzed
                        </div>
                        <div>
                            <span className="font-medium">{analysis.summary.avg_load_time.toFixed(2)}s</span> avg load time
                        </div>
                        <div>
                            <span className="font-medium">{analysis.summary.mobile_friendly_pages}</span> mobile-friendly
                        </div>
                        <div>
                            <span className="font-medium">{analysis.summary.total_words}</span> total words
                        </div>
                    </div>
                )}

                {/* Action Buttons */}
                <div className="flex items-center gap-2 pt-4 border-t border-gray-200 dark:border-gray-700">
                    {analysis.status === 'completed' && (
                        <>
                            <button
                                onClick={() => onView(analysis)}
                                className="flex-1 px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white text-sm font-medium rounded-lg transition-colors"
                            >
                                View Report
                            </button>
                            {onExport && (
                                <div className="relative group">
                                    <button className="px-3 py-2 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 text-gray-700 dark:text-gray-300 rounded-lg transition-colors">
                                        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                                        </svg>
                                    </button>
                                    <div className="absolute right-0 top-full mt-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all z-10">
                                        <button
                                            onClick={() => onExport(analysis.id, 'pdf')}
                                            className="block w-full text-left px-3 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 first:rounded-t-lg"
                                        >
                                            Export PDF
                                        </button>
                                        <button
                                            onClick={() => onExport(analysis.id, 'csv')}
                                            className="block w-full text-left px-3 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
                                        >
                                            Export CSV
                                        </button>
                                        <button
                                            onClick={() => onExport(analysis.id, 'json')}
                                            className="block w-full text-left px-3 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 last:rounded-b-lg"
                                        >
                                            Export JSON
                                        </button>
                                    </div>
                                </div>
                            )}
                        </>
                    )}

                    {analysis.status === 'analyzing' && onPause && (
                        <button
                            onClick={() => onPause(analysis.id)}
                            className="px-4 py-2 bg-yellow-500 hover:bg-yellow-600 text-white text-sm font-medium rounded-lg transition-colors"
                        >
                            Pause
                        </button>
                    )}

                    {analysis.status === 'paused' && onResume && (
                        <button
                            onClick={() => onResume(analysis.id)}
                            className="px-4 py-2 bg-green-500 hover:bg-green-600 text-white text-sm font-medium rounded-lg transition-colors"
                        >
                            Resume
                        </button>
                    )}

                    <button
                        onClick={() => onDelete(analysis.id)}
                        className="px-3 py-2 text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
                        title="Delete analysis"
                    >
                        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                        </svg>
                    </button>
                </div>
            </div>
        </div>
    );
};