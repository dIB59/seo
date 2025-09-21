// components/SeoReportTable.tsx
import React, { useState, useMemo } from 'react';
import { PageAnalysis, SeoIssue } from '../types/seo';

interface SeoReportTableProps {
    pages: PageAnalysis[];
    analysisUrl: string;
}

type SortField = keyof PageAnalysis | 'issues_count';
type SortDirection = 'asc' | 'desc';
type FilterTab = 'all' | 'issues' | 'images' | 'internal' | 'external' | 'redirects';

export const SeoReportTable: React.FC<SeoReportTableProps> = ({ pages, analysisUrl }) => {
    const [sortField, setSortField] = useState<SortField>('url');
    const [sortDirection, setSortDirection] = useState<SortDirection>('asc');
    const [filterTab, setFilterTab] = useState<FilterTab>('all');
    const [searchQuery, setSearchQuery] = useState('');
    const [selectedRows, setSelectedRows] = useState<Set<string>>(new Set());

    // Filter pages based on active tab and search
    const filteredPages = useMemo(() => {
        let filtered = pages;

        // Apply tab filter
        switch (filterTab) {
            case 'issues':
                filtered = pages.filter(page => page.issues.length > 0);
                break;
            case 'images':
                filtered = pages.filter(page => page.image_count > 0);
                break;
            case 'internal':
                filtered = pages.filter(page => page.internal_links > 0);
                break;
            case 'external':
                filtered = pages.filter(page => page.external_links > 0);
                break;
            case 'redirects':
                filtered = pages.filter(page => page.status_code >= 300 && page.status_code < 400);
                break;
            default:
                filtered = pages;
        }

        // Apply search filter
        if (searchQuery) {
            filtered = filtered.filter(page =>
                page.url.toLowerCase().includes(searchQuery.toLowerCase()) ||
                page.title?.toLowerCase().includes(searchQuery.toLowerCase()) ||
                page.meta_description?.toLowerCase().includes(searchQuery.toLowerCase())
            );
        }

        return filtered;
    }, [pages, filterTab, searchQuery]);

    // Sort pages
    const sortedPages = useMemo(() => {
        return [...filteredPages].sort((a, b) => {
            let aValue: any, bValue: any;

            if (sortField === 'issues_count') {
                aValue = a.issues.length;
                bValue = b.issues.length;
            } else {
                aValue = a[sortField];
                bValue = b[sortField];
            }

            if (typeof aValue === 'string' && typeof bValue === 'string') {
                aValue = aValue.toLowerCase();
                bValue = bValue.toLowerCase();
            }

            if (aValue < bValue) return sortDirection === 'asc' ? -1 : 1;
            if (aValue > bValue) return sortDirection === 'asc' ? 1 : -1;
            return 0;
        });
    }, [filteredPages, sortField, sortDirection]);

    const handleSort = (field: SortField) => {
        if (field === sortField) {
            setSortDirection(prev => prev === 'asc' ? 'desc' : 'asc');
        } else {
            setSortField(field);
            setSortDirection('asc');
        }
    };

    const toggleRowSelection = (url: string) => {
        const newSelected = new Set(selectedRows);
        if (newSelected.has(url)) {
            newSelected.delete(url);
        } else {
            newSelected.add(url);
        }
        setSelectedRows(newSelected);
    };

    const selectAllRows = () => {
        if (selectedRows.size === sortedPages.length) {
            setSelectedRows(new Set());
        } else {
            setSelectedRows(new Set(sortedPages.map(page => page.url)));
        }
    };

    const getStatusBadge = (statusCode: number) => {
        if (statusCode >= 200 && statusCode < 300) {
            return <span className="px-2 py-1 bg-green-100 text-green-800 text-xs rounded-full">OK</span>;
        } else if (statusCode >= 300 && statusCode < 400) {
            return <span className="px-2 py-1 bg-yellow-100 text-yellow-800 text-xs rounded-full">Redirect</span>;
        } else if (statusCode >= 400) {
            return <span className="px-2 py-1 bg-red-100 text-red-800 text-xs rounded-full">Error</span>;
        }
        return statusCode;
    };

    const getIssuesPriority = (issues: SeoIssue[]) => {
        const critical = issues.filter(i => i.type === 'critical').length;
        const warnings = issues.filter(i => i.type === 'warning').length;
        const suggestions = issues.filter(i => i.type === 'suggestion').length;

        if (critical > 0) return <span className="text-red-600 font-semibold">{critical} Critical</span>;
        if (warnings > 0) return <span className="text-yellow-600 font-semibold">{warnings} Warnings</span>;
        if (suggestions > 0) return <span className="text-blue-600">{suggestions} Suggestions</span>;
        return <span className="text-green-600">No Issues</span>;
    };

    const SortableHeader: React.FC<{ field: SortField; children: React.ReactNode }> = ({ field, children }) => (
        <th
            className="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700 select-none"
            onClick={() => handleSort(field)}
        >
            <div className="flex items-center space-x-1">
                <span>{children}</span>
                {sortField === field && (
                    <svg className={`w-3 h-3 transform ${sortDirection === 'desc' ? 'rotate-180' : ''}`} fill="currentColor" viewBox="0 0 20 20">
                        <path fillRule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clipRule="evenodd" />
                    </svg>
                )}
            </div>
        </th>
    );

    const tabCounts = {
        all: pages.length,
        issues: pages.filter(p => p.issues.length > 0).length,
        images: pages.filter(p => p.image_count > 0).length,
        internal: pages.filter(p => p.internal_links > 0).length,
        external: pages.filter(p => p.external_links > 0).length,
        redirects: pages.filter(p => p.status_code >= 300 && p.status_code < 400).length,
    };

    return (
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg">
            {/* Header Controls */}
            <div className="p-6 border-b border-gray-200 dark:border-gray-700">
                <div className="flex items-center justify-between mb-4">
                    <h2 className="text-2xl font-semibold text-gray-900 dark:text-white">
                        SEO Analysis Results
                    </h2>
                    <div className="flex items-center gap-2">
                        <span className="text-sm text-gray-500 dark:text-gray-400">
                            {selectedRows.size} of {sortedPages.length} selected
                        </span>
                        <button className="px-3 py-1 bg-blue-500 hover:bg-blue-600 text-white text-sm rounded transition-colors">
                            Export Selected
                        </button>
                    </div>
                </div>

                {/* Search */}
                <div className="mb-4">
                    <div className="relative">
                        <input
                            type="text"
                            placeholder="Search URLs, titles, or descriptions..."
                            value={searchQuery}
                            onChange={(e) => setSearchQuery(e.target.value)}
                            className="w-full px-4 py-2 pl-10 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                        />
                        <svg className="absolute left-3 top-2.5 w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                        </svg>
                    </div>
                </div>

                {/* Filter Tabs */}
                <div className="flex space-x-1 bg-gray-100 dark:bg-gray-700 p-1 rounded-lg">
                    {Object.entries(tabCounts).map(([tab, count]) => (
                        <button
                            key={tab}
                            onClick={() => setFilterTab(tab as FilterTab)}
                            className={`px-4 py-2 text-sm font-medium rounded-md transition-colors ${filterTab === tab
                                    ? 'bg-white dark:bg-gray-600 text-gray-900 dark:text-white shadow-sm'
                                    : 'text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white'
                                }`}
                        >
                            {tab.charAt(0).toUpperCase() + tab.slice(1)} ({count})
                        </button>
                    ))}
                </div>
            </div>

            {/* Table */}
            <div className="overflow-x-auto">
                <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                    <thead className="bg-gray-50 dark:bg-gray-700">
                        <tr>
                            <th className="px-4 py-3 text-left">
                                <input
                                    type="checkbox"
                                    checked={selectedRows.size === sortedPages.length && sortedPages.length > 0}
                                    onChange={selectAllRows}
                                    className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                />
                            </th>
                            <SortableHeader field="url">URL</SortableHeader>
                            <SortableHeader field="status_code">Status</SortableHeader>
                            <SortableHeader field="title">Title</SortableHeader>
                            <SortableHeader field="meta_description">Meta Description</SortableHeader>
                            <SortableHeader field="h1_count">H1</SortableHeader>
                            <SortableHeader field="word_count">Words</SortableHeader>
                            <SortableHeader field="load_time">Load Time</SortableHeader>
                            <SortableHeader field="image_count">Images</SortableHeader>
                            <SortableHeader field="internal_links">Internal</SortableHeader>
                            <SortableHeader field="external_links">External</SortableHeader>
                            <SortableHeader field="issues_count">Issues</SortableHeader>
                        </tr>
                    </thead>
                    <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                        {sortedPages.map((page) => (
                            <tr
                                key={page.url}
                                className={`hover:bg-gray-50 dark:hover:bg-gray-700 ${selectedRows.has(page.url) ? 'bg-blue-50 dark:bg-blue-900/20' : ''
                                    }`}
                            >
                                <td className="px-4 py-3">
                                    <input
                                        type="checkbox"
                                        checked={selectedRows.has(page.url)}
                                        onChange={() => toggleRowSelection(page.url)}
                                        className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                    />
                                </td>
                                <td className="px-4 py-3 text-sm">
                                    <a
                                        href={page.url}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        className="text-blue-600 dark:text-blue-400 hover:underline max-w-xs truncate block"
                                        title={page.url}
                                    >
                                        {page.url}
                                    </a>
                                </td>
                                <td className="px-4 py-3">{getStatusBadge(page.status_code)}</td>
                                <td className="px-4 py-3 text-sm max-w-xs truncate" title={page.title}>
                                    {page.title || <span className="text-gray-400 italic">No title</span>}
                                </td>
                                <td className="px-4 py-3 text-sm max-w-xs truncate" title={page.meta_description}>
                                    {page.meta_description || <span className="text-gray-400 italic">No description</span>}
                                </td>
                                <td className="px-4 py-3 text-sm text-center">
                                    <span className={page.h1_count === 1 ? 'text-green-600' : page.h1_count === 0 ? 'text-red-600' : 'text-yellow-600'}>
                                        {page.h1_count}
                                    </span>
                                </td>
                                <td className="px-4 py-3 text-sm text-right">{page.word_count.toLocaleString()}</td>
                                <td className="px-4 py-3 text-sm text-right">
                                    <span className={page.load_time > 3 ? 'text-red-600' : page.load_time > 1.5 ? 'text-yellow-600' : 'text-green-600'}>
                                        {page.load_time.toFixed(2)}s
                                    </span>
                                </td>
                                <td className="px-4 py-3 text-sm text-center">
                                    {page.image_count}
                                    {page.images_without_alt > 0 && (
                                        <span className="ml-1 text-red-600" title={`${page.images_without_alt} without alt text`}>
                                            ⚠
                                        </span>
                                    )}
                                </td>
                                <td className="px-4 py-3 text-sm text-center">{page.internal_links}</td>
                                <td className="px-4 py-3 text-sm text-center">{page.external_links}</td>
                                <td className="px-4 py-3 text-sm">{getIssuesPriority(page.issues)}</td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>

            {sortedPages.length === 0 && (
                <div className="text-center py-12">
                    <p className="text-gray-500 dark:text-gray-400">
                        No pages found matching the current filters.
                    </p>
                </div>
            )}

            {/* Footer */}
            <div className="px-6 py-4 bg-gray-50 dark:bg-gray-700 border-t border-gray-200 dark:border-gray-600">
                <div className="flex items-center justify-between text-sm text-gray-600 dark:text-gray-400">
                    <span>
                        Showing {sortedPages.length} of {pages.length} pages
                    </span>
                    <span>
                        Base URL: {analysisUrl}
                    </span>
                </div>
            </div>
        </div>
    );
};