import { useMemo, useEffect, useState } from "react"
import type { CompleteAnalysisResult } from "@/src/lib/types"
import { normalizeUrl, resolveInternalUrl } from "./url-utils"
import { calculateNodeDegrees } from "./node-utils"

export const useGraphData = (data: CompleteAnalysisResult, selectedNodeId: string | null) => {
    return useMemo(() => {
        type GraphNode = any
        type GraphLink = any

        const nodes: GraphNode[] = []
        const links: GraphLink[] = []

        const validUrls = new Map<string, string>()
        data.pages.forEach(page => {
            validUrls.set(normalizeUrl(page.url), page.url)
        })

        const { inDegree, outDegree } = calculateNodeDegrees(data.pages, validUrls)

        data.pages.forEach(page => {
            const issuesForPage = data.issues.filter(i => i.page_url === page.url)
            const baseColor = issuesForPage.some(i => i.severity === "critical") ? '#f14444ff' : issuesForPage.some(i => i.severity === "warning") ? '#e8aa3fff' : '#46c773ff'

            nodes.push({
                id: page.url,
                url: page.url,
                title: page.title || "No Title",
                status: page.status_code,
                issueCount: issuesForPage.length,
                inDegree: inDegree.get(page.url) || 0,
                outDegree: outDegree.get(page.url) || 0,
                color: baseColor
            })
        })

        data.pages.forEach(page => {
            if (!page.detailed_links) return

            page.detailed_links.forEach(link => {
                if (!link.is_internal) return

                const targetUrl = resolveInternalUrl(link.href, page.url, validUrls)
                if (!targetUrl) return

                const targetPage = data.pages.find(p => p.url === targetUrl)
                const isBroken = targetPage?.status_code ? targetPage.status_code >= 400 : false

                links.push({ source: page.url, target: targetUrl, isBroken })
            })
        })

        if (selectedNodeId) {
            const filteredLinks = links.filter(
                link => link.source === selectedNodeId || link.target === selectedNodeId
            )

            const connectedNodeIds = new Set([selectedNodeId])
            filteredLinks.forEach(link => {
                connectedNodeIds.add(link.source)
                connectedNodeIds.add(link.target)
            })

            nodes.forEach(node => {
                if (!connectedNodeIds.has(node.id)) node.color = '#666666ff'
            })

            return { nodes, links: filteredLinks }
        }

        return { nodes, links }
    }, [data, selectedNodeId])
}

export const useContainerDimensions = (containerRef: React.RefObject<HTMLDivElement | null>) => {
    const [dimensions, setDimensions] = useState({ width: 800, height: 600 })

    useEffect(() => {
        if (!containerRef.current) return

        const observer = new ResizeObserver((entries) => {
            if (!entries[0]) return
            const { width, height } = entries[0].contentRect
            setDimensions({ width, height })
        })

        observer.observe(containerRef.current)

        return () => observer.disconnect()
    }, [containerRef])

    return dimensions
}
