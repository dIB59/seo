// app/page.tsx
"use client";
import React, { useState } from 'react';
import { useSeoAnalysis } from './hooks/useSeoAnalysis';
import { UrlInput } from './components/UrlInput';
import { AnalysisCard } from './components/AnalysisCard';
import { AnalysisResult } from './types/seo';
import { ReportPage } from './components/ReportPage';

type ViewMode = 'home' | 'report';

export default function Home() {
  const [viewMode, setViewMode] = useState<ViewMode>('home');
  const [selectedAnalysis, setSelectedAnalysis] = useState<AnalysisResult | null>(null);

  const {
    currentAnalysis,
    recentAnalyses,
    isAnalyzing,
    error,
    isLoading,
    startAnalysis,
    pauseAnalysis,
    resumeAnalysis,
    deleteAnalysis,
    exportReport,
    clearError,
    refreshAnalyses,
  } = useSeoAnalysis();

  const handleViewReport = (analysis: AnalysisResult) => {
    setSelectedAnalysis(analysis);
    setViewMode('report');
  };

  const handleBackToHome = () => {
    setViewMode('home');
    setSelectedAnalysis(null);
  };

  const handleExport = async (analysisId: string, format: 'pdf' | 'csv' | 'json') => {
    try {
      const filePath = await exportReport(analysisId, format);
      console.log(`Report exported to: ${filePath}`);
    } catch (error) {
      console.error('Export failed:', error);
    }
  };

  const isViewingReport = viewMode === 'report' && selectedAnalysis;
  const hasAnalysisInProgress = currentAnalysis && isAnalyzing;
  const hasRecentAnalyses = recentAnalyses.length > 0;
  const showEmptyState = !hasRecentAnalyses && !currentAnalysis && !isLoading;

  if (isViewingReport) {
    return (
      <ReportPage
        analysis={selectedAnalysis}
        onBack={handleBackToHome}
      />
    );
  }

  // Components
  const Header = () => (
    <header className="text-center mb-12">
      <div className="flex items-center justify-center gap-3 mb-4">

        <h1 className="text-4xl font-bold bg-clip-text">
          SEO Analyzer
        </h1>
      </div>

    </header>
  );

  const AnalysisProgress = () => (
    <div className="bg-blue-50 dark:bg-blue-900/20 rounded-2xl p-6 border border-blue-200 dark:border-blue-800">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-blue-900 dark:text-blue-100">
          Analyzing {currentAnalysis!.url}
        </h3>
        <div className="flex items-center gap-2">
          <div className="flex items-center text-blue-600 dark:text-blue-400">
            <svg className="animate-spin w-5 h-5 mr-2" fill="none" viewBox="0 0 24 24">
              <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
              <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            <span className="text-sm font-medium">
              {currentAnalysis!.status === 'paused' ? 'Paused' : 'In Progress'}
            </span>
          </div>
          {currentAnalysis!.status === 'analyzing' && (
            <button
              onClick={() => pauseAnalysis(currentAnalysis!.id)}
              className="px-3 py-1 bg-yellow-500 hover:bg-yellow-600 text-white text-sm rounded-md transition-colors"
            >
              Pause
            </button>
          )}
          {currentAnalysis!.status === 'paused' && (
            <button
              onClick={() => resumeAnalysis(currentAnalysis!.id)}
              className="px-3 py-1 bg-green-500 hover:bg-green-600 text-white text-sm rounded-md transition-colors"
            >
              Resume
            </button>
          )}
        </div>
      </div>

      <div className="space-y-3">
        <div className="flex justify-between text-sm text-blue-700 dark:text-blue-300">
          <span>Progress</span>
          <span>{Math.round(currentAnalysis!.progress)}%</span>
        </div>
        <div className="w-full bg-blue-200 dark:bg-blue-800 rounded-full h-3">
          <div
            className="h-3 rounded-full transition-all duration-500"
            style={{ width: `${currentAnalysis!.progress}%` }}
          />
        </div>
        <div className="flex justify-between text-sm text-blue-600 dark:text-blue-400">
          <span>Pages analyzed: {currentAnalysis!.analyzed_pages}</span>
          <span>Total pages: {currentAnalysis!.total_pages || 'Discovering...'}</span>
        </div>
      </div>
    </div>
  );

  const RecentAnalysesSection = () => (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-semibold text-gray-800 dark:text-white">
          Recent Analyses
        </h2>
        <button
          onClick={refreshAnalyses}
          disabled={isLoading}
          className="px-4 py-2 text-sm text-blue-600 dark:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded-lg transition-colors disabled:opacity-50"
        >
          {isLoading ? (
            <svg className="animate-spin w-4 h-4" fill="none" viewBox="0 0 24 24">
              <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
              <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
            </svg>
          ) : (
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
          )}
          <span className="ml-1">Refresh</span>
        </button>
      </div>

      <div className="grid gap-6">
        {recentAnalyses.map((analysis) => (
          <AnalysisCard
            key={analysis.id}
            analysis={analysis}
            onView={handleViewReport}
            onDelete={deleteAnalysis}
            onPause={pauseAnalysis}
            onResume={resumeAnalysis}
            onExport={handleExport}
          />
        ))}
      </div>
    </div>
  );

  const EmptyState = () => (
    <div className="text-center py-12">
      <div className="w-24 h-24 bg-gray-100 dark:bg-gray-700 rounded-full flex items-center justify-center mx-auto mb-4">
        <svg className="w-12 h-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
        </svg>
      </div>
      <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
        No analyses yet
      </h3>
      <p className="text-gray-500 dark:text-gray-400">
        Start your first SEO analysis by entering a website URL above.
      </p>
    </div>
  );

  // Main home page render
  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 via-white to-purple-50 dark:from-gray-900 dark:via-gray-800 dark:to-gray-900">
      <div className="container mx-auto px-4 py-8">
        <Header />

        <div className="max-w-4xl mx-auto space-y-8">
          <UrlInput
            onStartAnalysis={startAnalysis}
            isAnalyzing={isAnalyzing}
            error={error}
            onClearError={clearError}
          />

          {hasAnalysisInProgress && <AnalysisProgress />}
          {hasRecentAnalyses && <RecentAnalysesSection />}
          {showEmptyState && <EmptyState />}
        </div>
      </div>
    </div>
  );
}