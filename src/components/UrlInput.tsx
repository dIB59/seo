// components/UrlInput.tsx
import React, { useState } from 'react';
import { AnalysisSettings } from '../types/seo';
import { seoUtils } from '../services/seoUtils';

interface UrlInputProps {
    onStartAnalysis: (url: string, settings?: Partial<AnalysisSettings>) => Promise<void>;
    isAnalyzing: boolean;
    error?: string | null;
    onClearError?: () => void;
}

export const UrlInput: React.FC<UrlInputProps> = ({
    onStartAnalysis,
    isAnalyzing,
    error,
    onClearError,
}) => {
    const [url, setUrl] = useState('');
    const [showSettings, setShowSettings] = useState(false);
    const [settings, setSettings] = useState<Partial<AnalysisSettings>>({
        max_pages: 100,
        include_external_links: true,
        check_images: true,
        mobile_analysis: true,
        lighthouse_analysis: false,
        delay_between_requests: 1000,
    });

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();

        if (!url.trim()) {
            return;
        }

        if (!seoUtils.isValidUrl(url)) {
            return;
        }

        await onStartAnalysis(url, settings);
    };

    const handleUrlChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        setUrl(e.target.value);
        if (error && onClearError) {
            onClearError();
        }
    };

    const isValidUrl = url.trim() ? seoUtils.isValidUrl(url) : true;

    return (
        <div className="bg-white dark:bg-gray-800 rounded-2xl shadow-xl p-8">
            <div className="flex items-center justify-between mb-6">
                <h2 className="text-2xl font-semibold text-gray-800 dark:text-white">
                    Analyze Your Website
                </h2>
                <button
                    onClick={() => setShowSettings(!showSettings)}
                    className="px-3 py-2 text-sm text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white transition-colors"
                >
                    <svg className="w-5 h-5 inline mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                    Settings
                </button>
            </div>

            <form onSubmit={handleSubmit} className="space-y-4">
                <div>
                    <label htmlFor="url" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                        Website URL
                    </label>
                    <div className="relative">
                        <input
                            type="url"
                            id="url"
                            value={url}
                            onChange={handleUrlChange}
                            placeholder="https://your-website.com"
                            disabled={isAnalyzing}
                            className={`w-full px-4 py-3 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400 transition-colors ${!isValidUrl
                                ? 'border-red-300 dark:border-red-600'
                                : 'border-gray-300 dark:border-gray-600'
                                }`}
                        />
                        <div className="absolute inset-y-0 right-0 flex items-center pr-3">
                            <svg className="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9v-9m0-9v9" />
                            </svg>
                        </div>
                    </div>
                    {!isValidUrl && (
                        <p className="mt-1 text-sm text-red-600 dark:text-red-400">
                            Please enter a valid URL (e.g., https://example.com)
                        </p>
                    )}
                </div>

                {/* Settings Panel */}
                {showSettings && (
                    <div className="p-4 bg-gray-50 dark:bg-gray-700 rounded-lg border border-gray-200 dark:border-gray-600">
                        <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-4">Analysis Settings</h3>

                        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <div>
                                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                    Maximum Pages to Analyze
                                </label>
                                <input
                                    type="number"
                                    min="1"
                                    max="1000"
                                    value={settings.max_pages}
                                    onChange={(e) => setSettings(prev => ({ ...prev, max_pages: parseInt(e.target.value) || 100 }))}
                                    className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                                />
                            </div>

                            <div>
                                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                    Delay Between Requests (ms)
                                </label>
                                <input
                                    type="number"
                                    min="100"
                                    max="10000"
                                    step="100"
                                    value={settings.delay_between_requests}
                                    onChange={(e) => setSettings(prev => ({ ...prev, delay_between_requests: parseInt(e.target.value) || 1000 }))}
                                    className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                                />
                            </div>
                        </div>

                        <div className="mt-4 space-y-3">
                            <label className="flex items-center">
                                <input
                                    type="checkbox"
                                    checked={settings.include_external_links}
                                    onChange={(e) => setSettings(prev => ({ ...prev, include_external_links: e.target.checked }))}
                                    className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                />
                                <span className="ml-2 text-sm text-gray-700 dark:text-gray-300">
                                    Analyze external links
                                </span>
                            </label>

                            <label className="flex items-center">
                                <input
                                    type="checkbox"
                                    checked={settings.check_images}
                                    onChange={(e) => setSettings(prev => ({ ...prev, check_images: e.target.checked }))}
                                    className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                />
                                <span className="ml-2 text-sm text-gray-700 dark:text-gray-300">
                                    Check image optimization
                                </span>
                            </label>

                            <label className="flex items-center">
                                <input
                                    type="checkbox"
                                    checked={settings.mobile_analysis}
                                    onChange={(e) => setSettings(prev => ({ ...prev, mobile_analysis: e.target.checked }))}
                                    className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                />
                                <span className="ml-2 text-sm text-gray-700 dark:text-gray-300">
                                    Mobile-friendly analysis
                                </span>
                            </label>

                            <label className="flex items-center">
                                <input
                                    type="checkbox"
                                    checked={settings.lighthouse_analysis}
                                    onChange={(e) => setSettings(prev => ({ ...prev, lighthouse_analysis: e.target.checked }))}
                                    className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                />
                                <span className="ml-2 text-sm text-gray-700 dark:text-gray-300">
                                    Run Lighthouse analysis (slower)
                                </span>
                            </label>
                        </div>
                    </div>
                )}

                {/* Error Message */}
                {error && (
                    <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-3">
                        <p className="text-red-700 dark:text-red-400 text-sm">{error}</p>
                    </div>
                )}

                {/* Submit Button */}
                <button
                    type="submit"
                    disabled={isAnalyzing || !url.trim() || !isValidUrl}
                    className={`w-full py-3 px-6 text-lg font-medium rounded-lg transition-all ${isAnalyzing || !url.trim() || !isValidUrl
                        ? 'bg-gray-300 dark:bg-gray-600 text-gray-500 dark:text-gray-400 cursor-not-allowed'
                        : ' hover:from-blue-600 hover:to-purple-700 text-white shadow-lg hover:shadow-xl transform hover:-translate-y-0.5'
                        }`}
                >
                    {isAnalyzing ? (
                        <div className="flex items-center justify-center">
                            <svg className="animate-spin w-5 h-5 mr-2" fill="none" viewBox="0 0 24 24">
                                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                            </svg>
                            Analyzing...
                        </div>
                    ) : (
                        'Start SEO Analysis'
                    )}
                </button>
            </form>
        </div>
    );
};