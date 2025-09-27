
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