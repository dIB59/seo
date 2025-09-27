// components/ReportPage.tsx
import React, { useState } from 'react';
import { AnalysisResult } from '../types/seo';
import { SeoReportTable } from './SeoReportTable';
import { seoUtils } from '../services/seoUtils';

interface ReportPageProps {
    analysis: AnalysisResult;
    onBack: () => void;
}

export const ReportPage: React.FC<ReportPageProps> = ({ analysis, onBack }) => {
    const [activeTab, setActiveTab] = useState<'overview' | 'pages' | 'issues'>('overview');

    const formatDate = (dateString: string) => {
        return new Date(dateString).toLocaleDateString('en-US', {
            year: 'numeric',
            month: 'long',
            day: 'numeric',
            hour: '2-digit',
            minute: '2-digit',
        });
    };

    const getScoreColor = (score: number) => {
        if (score >= 80) return 'text-green-600 dark:text-green-400';
        if (score >= 60) return 'text-yellow-600 dark:text-yellow-400';
        return 'text-red-600 dark:text-red-400';
    };

    const OverviewTab = () => (
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            {/* SEO Score Card */}
            <div className="lg:col-span-1">
                <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
                    <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">SEO Score</h3>
                    <div className="text-center">
                        <div className={`text-6xl font-bold ${getScoreColor(analysis.summary.seo_score)}`}>
                            {analysis.summary.seo_score}
                        </div>
                        <div className="text-gray-500 dark:text-gray-400 mt-2">out of 100</div>
                        <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-3 mt-4">
                            <div
                                className={`h-3 rounded-full transition-all ${analysis.summary.seo_score >= 80 ? 'bg-green-500' :
                                    analysis.summary.seo_score >= 60 ? 'bg-yellow-500' : 'bg-red-500'
                                    }`}
                                style={{ width: `${analysis.summary.seo_score}%` }}
                            />
                        </div>
                    </div>
                </div>
            </div>

            {/* Key Metrics */}
            <div className="lg:col-span-2">
                <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
                    <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">Key Metrics</h3>
                    <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                        <div className="text-center">
                            <div className="text-2xl font-bold text-blue-600 dark:text-blue-400">
                                {analysis.total_pages}
                            </div>
                            <div className="text-sm text-gray-600 dark:text-gray-400">Pages Analyzed</div>
                        </div>
                        <div className="text-center">
                            <div className="text-2xl font-bold text-green-600 dark:text-green-400">
                                {analysis.summary.mobile_friendly_pages}
                            </div>
                            <div className="text-sm text-gray-600 dark:text-gray-400">Mobile Friendly</div>
                        </div>
                        <div className="text-2xl font-bold text-yellow-600 dark:text-yellow-400 text-center">
                            {analysis.summary.avg_load_time.toFixed(2)}s
                        </div>
                        <div className="text-sm text-gray-600 dark:text-gray-400 text-center">Avg Load Time</div>
                        <div className="text-center">
                            <div className="text-2xl font-bold text-purple-600 dark:text-purple-400">
                                {analysis.summary.total_words.toLocaleString()}
                            </div>
                            <div className="text-sm text-gray-600 dark:text-gray-400">Total Words</div>
                        </div>
                    </div>
                </div>
            </div>

            {/* Issues Summary */}
            <div className="lg:col-span-3">
                <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
                    <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">Issues Summary</h3>
                    <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                        <div className="flex items-center p-4 bg-red-50 dark:bg-red-900/20 rounded-lg">
                            <div className="w-12 h-12 bg-red-100 dark:bg-red-900/40 rounded-full flex items-center justify-center mr-4">
                                <svg className="w-6 h-6 text-red-600 dark:text-red-400" fill="currentColor" viewBox="0 0 20 20">
                                    <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
                                </svg>
                            </div>
                            <div>
                                <div className="text-2xl font-bold text-red-600 dark:text-red-400">
                                    {analysis.issues.critical}
                                </div>
                                <div className="text-red-700 dark:text-red-300 font-medium">Critical Issues</div>
                                <div className="text-red-600 dark:text-red-400 text-sm">Immediate attention required</div>
                            </div>
                        </div>

                        <div className="flex items-center p-4 bg-yellow-50 dark:bg-yellow-900/20 rounded-lg">
                            <div className="w-12 h-12 bg-yellow-100 dark:bg-yellow-900/40 rounded-full flex items-center justify-center mr-4">
                                <svg className="w-6 h-6 text-yellow-600 dark:text-yellow-400" fill="currentColor" viewBox="0 0 20 20">
                                    <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
                                </svg>
                            </div>
                            <div>
                                <div className="text-2xl font-bold text-yellow-600 dark:text-yellow-400">
                                    {analysis.issues.warnings}
                                </div>
                                <div className="text-yellow-700 dark:text-yellow-300 font-medium">Warnings</div>
                                <div className="text-yellow-600 dark:text-yellow-400 text-sm">Should be addressed</div>
                            </div>
                        </div>

                        <div className="flex items-center p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
                            <div className="w-12 h-12 bg-blue-100 dark:bg-blue-900/40 rounded-full flex items-center justify-center mr-4">
                                <svg className="w-6 h-6 text-blue-600 dark:text-blue-400" fill="currentColor" viewBox="0 0 20 20">
                                    <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clipRule="evenodd" />
                                </svg>
                            </div>
                            <div>
                                <div className="text-2xl font-bold text-blue-600 dark:text-blue-400">
                                    {analysis.issues.suggestions}
                                </div>
                                <div className="text-blue-700 dark:text-blue-300 font-medium">Suggestions</div>
                                <div className="text-blue-600 dark:text-blue-400 text-sm">Optimization opportunities</div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            {/* Technical Status */}
            <div className="lg:col-span-3">
                <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
                    <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">Technical Status</h3>
                    <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                        <div className="flex items-center justify-between p-4 border border-gray-200 dark:border-gray-700 rounded-lg">
                            <span className="font-medium text-gray-900 dark:text-white">SSL Certificate</span>
                            {analysis.ssl_certificate ? (
                                <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400">
                                    <svg className="w-3 h-3 mr-1" fill="currentColor" viewBox="0 0 20 20">
                                        <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
                                    </svg>
                                    Active
                                </span>
                            ) : (
                                <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400">
                                    <svg className="w-3 h-3 mr-1" fill="currentColor" viewBox="0 0 20 20">
                                        <path fillRule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clipRule="evenodd" />
                                    </svg>
                                    Missing
                                </span>
                            )}
                        </div>

                        <div className="flex items-center justify-between p-4 border border-gray-200 dark:border-gray-700 rounded-lg">
                            <span className="font-medium text-gray-900 dark:text-white">Sitemap.xml</span>
                            {analysis.sitemap_found ? (
                                <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400">
                                    <svg className="w-3 h-3 mr-1" fill="currentColor" viewBox="0 0 20 20">
                                        <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
                                    </svg>
                                    Found
                                </span>
                            ) : (
                                <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400">
                                    <svg className="w-3 h-3 mr-1" fill="currentColor" viewBox="0 0 20 20">
                                        <path fillRule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clipRule="evenodd" />
                                    </svg>
                                    Not Found
                                </span>
                            )}
                        </div>

                        <div className="flex items-center justify-between p-4 border border-gray-200 dark:border-gray-700 rounded-lg">
                            <span className="font-medium text-gray-900 dark:text-white">Robots.txt</span>
                            {analysis.robots_txt_found ? (
                                <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400">
                                    <svg className="w-3 h-3 mr-1" fill="currentColor" viewBox="0 0 20 20">
                                        <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
                                    </svg>
                                    Found
                                </span>
                            ) : (
                                <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400">
                                    <svg className="w-3 h-3 mr-1" fill="currentColor" viewBox="0 0 20 20">
                                        <path fillRule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clipRule="evenodd" />
                                    </svg>
                                    Not Found
                                </span>
                            )}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );

    const IssuesTab = () => {
        const allIssues = analysis.pages.flatMap(page =>
            page.issues.map(issue => ({ ...issue, page_title: page.title }))
        );

        const issuesByType = {
            critical: allIssues.filter(issue => issue.type === 'critical'),
            warning: allIssues.filter(issue => issue.type === 'warning'),
            suggestion: allIssues.filter(issue => issue.type === 'suggestion'),
        };

        return (
            <div className="space-y-6">
                {(['critical', 'warning', 'suggestion'] as const).map(type => (
                    <div key={type} className="bg-white dark:bg-gray-800 rounded-lg shadow">
                        <div className={`px-6 py-4 border-b border-gray-200 dark:border-gray-700 ${type === 'critical' ? 'bg-red-50 dark:bg-red-900/20' :
                            type === 'warning' ? 'bg-yellow-50 dark:bg-yellow-900/20' :
                                'bg-blue-50 dark:bg-blue-900/20'
                            }`}>
                            <h3 className={`text-lg font-semibold ${type === 'critical' ? 'text-red-900 dark:text-red-100' :
                                type === 'warning' ? 'text-yellow-900 dark:text-yellow-100' :
                                    'text-blue-900 dark:text-blue-100'
                                }`}>
                                {type.charAt(0).toUpperCase() + type.slice(1)} Issues ({issuesByType[type].length})
                            </h3>
                        </div>
                        <div className="divide-y divide-gray-200 dark:divide-gray-700">
                            {issuesByType[type].map((issue) => (
                                <div key={issue.id} className="p-6">
                                    <div className="flex justify-between items-start mb-2">
                                        <h4 className="text-lg font-medium text-gray-900 dark:text-white">
                                            {issue.title}
                                        </h4>
                                        <span className={`px-2 py-1 rounded-full text-xs font-medium ${type === 'critical' ? 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400' :
                                            type === 'warning' ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-400' :
                                                'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-400'
                                            }`}>
                                            {type.toUpperCase()}
                                        </span>
                                    </div>
                                    <p className="text-gray-600 dark:text-gray-400 mb-3">
                                        {issue.description}
                                    </p>
                                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                                        <div>
                                            <span className="font-medium text-gray-500 dark:text-gray-400">Affected URL:</span>
                                            <div className="mt-1">
                                                <a
                                                    href={issue.page_url}
                                                    target="_blank"
                                                    rel="noopener noreferrer"
                                                    className="text-blue-600 dark:text-blue-400 hover:underline break-all"
                                                >
                                                    {issue.page_url}
                                                </a>
                                            </div>
                                        </div>
                                        <div>
                                            <span className="font-medium text-gray-500 dark:text-gray-400">Recommendation:</span>
                                            <div className="mt-1 text-gray-900 dark:text-white">
                                                {issue.recommendation}
                                            </div>
                                        </div>
                                    </div>
                                    {issue.element && (
                                        <div className="mt-3">
                                            <span className="font-medium text-gray-500 dark:text-gray-400">Element:</span>
                                            <div className="mt-1 font-mono text-sm bg-gray-100 dark:bg-gray-900 p-2 rounded">
                                                {issue.element}
                                            </div>
                                        </div>
                                    )}
                                </div>
                            ))}
                            {issuesByType[type].length === 0 && (
                                <div className="p-6 text-center text-gray-500 dark:text-gray-400">
                                    No {type} issues found. Great job!
                                </div>
                            )}
                        </div>
                    </div>
                ))}
            </div>
        );
    };

    return (
        <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
            {/* Header */}
            <div className="bg-white dark:bg-gray-800 shadow">
                <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                    <div className="flex items-center justify-between h-16">
                        <div className="flex items-center">
                            <button
                                onClick={onBack}
                                className="flex items-center text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 mr-4"
                            >
                                <svg className="w-5 h-5 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
                                </svg>
                                Back
                            </button>
                            <div>
                                <h1 className="text-xl font-semibold text-gray-900 dark:text-white">
                                    SEO Report: {seoUtils.formatUrl(analysis.url)}
                                </h1>
                                <p className="text-sm text-gray-500 dark:text-gray-400">
                                    Completed {formatDate(analysis.completed_at || analysis.started_at)} •
                                    Duration: {seoUtils.formatDuration(analysis.started_at, analysis.completed_at)}
                                </p>
                            </div>
                        </div>
                        <div className="flex items-center space-x-4">
                            <button className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors">
                                Export PDF
                            </button>
                            <button className="px-4 py-2 border border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-lg transition-colors">
                                Export CSV
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            {/* Navigation Tabs */}
            <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
                <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                    <nav className="flex space-x-8">
                        {[
                            { key: 'overview', label: 'Overview', count: null },
                            { key: 'pages', label: 'All Pages', count: analysis.total_pages },
                            { key: 'issues', label: 'Issues', count: analysis.issues.critical + analysis.issues.warnings + analysis.issues.suggestions },
                        ].map(tab => (
                            <button
                                key={tab.key}
                                onClick={() => setActiveTab(tab.key as any)}
                                className={`py-4 px-1 border-b-2 font-medium text-sm transition-colors ${activeTab === tab.key
                                    ? 'border-blue-500 text-blue-600 dark:text-blue-400'
                                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400 dark:hover:text-gray-300'
                                    }`}
                            >
                                {tab.label}
                                {tab.count !== null && (
                                    <span className="ml-2 bg-gray-100 dark:bg-gray-700 text-gray-900 dark:text-gray-300 py-0.5 px-2 rounded-full text-xs">
                                        {tab.count}
                                    </span>
                                )}
                            </button>
                        ))}
                    </nav>
                </div>
            </div>

            {/* Content */}
            <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
                {activeTab === 'overview' && <OverviewTab />}
                {activeTab === 'pages' && <SeoReportTable pages={analysis.pages} analysisUrl={analysis.url} />}
                {activeTab === 'issues' && <IssuesTab />}
            </div>
        </div>
    );
};