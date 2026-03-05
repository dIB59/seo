export const normalizeUrl = (url: string): string => {
    try {
        const parsed = new URL(url)
        return (parsed.origin + parsed.pathname).replace(/\/$/, "")
    } catch {
        return url.replace(/\/$/, "")
    }
}

export const resolveInternalUrl = (href: string, baseUrl: string, validUrls: Map<string, string>): string | null => {
    const normalized = normalizeUrl(href)
    let targetUrl = validUrls.get(normalized)

    if (!targetUrl && !href.startsWith('http')) {
        try {
            const base = new URL(baseUrl)
            const absoluteUrl = new URL(href, base.origin).href
            targetUrl = validUrls.get(normalizeUrl(absoluteUrl))
        } catch {
            return null
        }
    }

    return targetUrl || null
}
