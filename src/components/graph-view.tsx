"use client"

import { useMemo, useRef, useState, useEffect } from "react"
import dynamic from "next/dynamic"
import { useTheme } from "next-themes"
import type { CompleteAnalysisResult } from "@/src/lib/types"
import { Card } from "@/src/components/ui/card"
import { ZoomIn, ZoomOut, RotateCcw, Settings2 } from "lucide-react"
import { Button } from "@/src/components/ui/button"
import { Slider } from "@/src/components/ui/slider"
import { Popover, PopoverContent, PopoverTrigger } from "@/src/components/ui/popover"
import { Label } from "@/src/components/ui/label"

// Dynamically import ForceGraph2D as it uses window/canvas
const ForceGraph2D = dynamic(() => import("react-force-graph-2d"), {
    ssr: false,
    loading: () => <div className="h-[600px] w-full flex items-center justify-center bg-muted/10 animate-pulse">Loading Graph...</div>
})

interface GraphViewProps {
    data: CompleteAnalysisResult
    onNodeClick?: (url: string) => void
}

interface GraphNode {
    id: string
    name: string
    val: number
    color: string
    status: number | null
    title: string
    issueCount: number
}

interface GraphLink {
    source: string
    target: string
    color: string
}

export function GraphView({ data, onNodeClick }: GraphViewProps) {
    const { resolvedTheme } = useTheme()
    const theme = resolvedTheme || 'dark'

    const fgRef = useRef<any>(null)
    const containerRef = useRef<HTMLDivElement>(null)
    const [dimensions, setDimensions] = useState({ width: 800, height: 600 })
    const [chargeStrength, setChargeStrength] = useState(-400)
    const [linkDistance, setLinkDistance] = useState(70)

    // Resize handler
    useEffect(() => {
        const updateDimensions = () => {
            if (containerRef.current) {
                setDimensions({
                    width: containerRef.current.clientWidth,
                    height: containerRef.current.clientHeight
                })
            }
        }

        window.addEventListener('resize', updateDimensions)
        updateDimensions()

        // Initial delay to ensure container is rendered
        setTimeout(updateDimensions, 100)

        return () => window.removeEventListener('resize', updateDimensions)
    }, [])

    const graphData = useMemo(() => {
        const nodes: GraphNode[] = []
        const links: GraphLink[] = []
        const nodeIds = new Set<string>()
        const inDegree = new Map<string, number>()
        const outDegree = new Map<string, number>()

        // 0. First Pass: Collect IDs and Calculate Degrees
        data.pages.forEach(page => {
            const id = page.url
            nodeIds.add(id)
            if (!inDegree.has(id)) inDegree.set(id, 0)
            if (!outDegree.has(id)) outDegree.set(id, 0)

            if (page.detailed_links) {
                page.detailed_links.forEach(link => {
                    if (link.is_internal) {
                        let targetId = link.href
                        // Normalize target logic (mirrors the logic used later)
                        // We assume specific normalization matches the second pass, 
                        // but for counting incoming connections rigorously we should verify target existence.
                        // However, we can just strictly count 'href' as target for now and filter later,
                        // OR better: do the normalization check here.

                        // Let's use a helper for normalization to be safe if we were extracting it, 
                        // but for now let's just do it inline or trust that exact matches are most important.
                        // Actually, to get ACCURATE in-degree for the VISUALIZED nodes, we must match the logic exactly.
                        // So let's do this: 
                        // 1. Collect all valid Node IDs first.
                    }
                })
            }
        })

        // Optimized Pass approach:
        // 1. Set of all page URLs (Potential Nodes)
        const validUrls = new Set(data.pages.map(p => p.url))

        // 2. Calculate connections
        data.pages.forEach(page => {
            if (page.detailed_links) {
                let currentOut = 0
                page.detailed_links.forEach(link => {
                    if (link.is_internal) {
                        let target = link.href
                        // Normalization check
                        if (!validUrls.has(target)) {
                            if (target.endsWith('/') && validUrls.has(target.slice(0, -1))) target = target.slice(0, -1)
                            else if (!target.endsWith('/') && validUrls.has(target + '/')) target = target + '/'
                        }

                        if (validUrls.has(target)) {
                            // Valid internal link
                            currentOut++
                            inDegree.set(target, (inDegree.get(target) || 0) + 1)
                        }
                    }
                })
                outDegree.set(page.url, currentOut)
            }
        })

        // 3. Create Nodes with topological insights
        data.pages.forEach((page) => {
            const issuesForPage = data.issues.filter(i => i.page_url === page.url)
            const isCritical = issuesForPage.some(i => i.issue_type === "Critical")
            const isWarning = issuesForPage.some(i => i.issue_type === "Warning")

            let color = "oklch(0.65 0.18 145)" // Success (Green)
            if (page.status_code && page.status_code >= 400) color = "oklch(0.55 0.2 25)" // Destructive (Red)
            else if (isCritical) color = "oklch(0.55 0.2 25)" // Critical (Red to Match Destructive)
            else if (isWarning) color = "oklch(0.75 0.15 85)" // Warning (Yellow)

            const id = page.url
            const incoming = inDegree.get(id) || 0
            const outgoing = outDegree.get(id) || 0

            // Sizing Logic: Base size + Logarithmic scaling based on In-degree
            // Base = 2 (minimum visibility)
            // Scaling: each connection adds weight, but diminishing returns.
            // Example: 0 in -> 2
            // Example: 10 in -> 2 + 10 * 0.5 ?? No that's huge. 
            // Log: 2 + Math.log(10+1) * 2 ~= 2 + 2.3*2 = 6.6
            // Example: 100 in -> 2 + Math.log(101) * 2 ~= 2 + 4.6*2 = 11.2 
            // This seems reasonable.
            const size = 2 + Math.log(incoming + 1) * 2

            nodes.push({
                id,
                name: page.url,
                val: size,
                color,
                status: page.status_code,
                title: page.title || "No Title",
                issueCount: issuesForPage.length,
                // Add specific stats for tooltip if we extend the node type, 
                // but we can just stuff it in 'name' or use a custom label generator.
                // We'll update the GraphNode interface to include these stats?
                // For now, let's keep it simple and just use 'val' for size.
                // We will cast to 'any' in the nodeLabel prop if needed or just use the object.
                inDegree: incoming,
                outDegree: outgoing
            } as any) // Casting as we're adding extra props for label
        })

        // 4. Create Links
        data.pages.forEach((page) => {
            if (page.detailed_links) {
                page.detailed_links.forEach(link => {
                    if (link.is_internal) {
                        let targetId = link.href;
                        // Use same normalization as above
                        if (!validUrls.has(targetId)) {
                            if (targetId.endsWith('/') && validUrls.has(targetId.slice(0, -1))) {
                                targetId = targetId.slice(0, -1);
                            } else if (!targetId.endsWith('/') && validUrls.has(targetId + '/')) {
                                targetId = targetId + '/';
                            }
                        }

                        if (validUrls.has(targetId)) {
                            // Link Insight: 
                            // If target has error (>=400), highlight the link as RED to show "Broken Path".
                            const targetPage = data.pages.find(p => p.url === targetId)
                            const isBrokenPath = targetPage && targetPage.status_code && targetPage.status_code >= 400

                            links.push({
                                source: page.url,
                                target: targetId,
                                color: isBrokenPath
                                    ? 'oklch(0.55 0.2 25)' // Red for broken paths
                                    : (theme === 'dark' ? 'rgba(255,255,255,0.5)' : 'rgba(0,0,0,0.5)')
                            })
                        }
                    }
                })
            }
        })

        return { nodes, links }
    }, [data, theme])


    // Configure forces
    useEffect(() => {
        if (fgRef.current) {
            fgRef.current.d3Force('charge').strength(chargeStrength).distanceMax(500);
            fgRef.current.d3Force('link').distance(linkDistance);

            // Re-heat simulation
            fgRef.current.d3ReheatSimulation();
        }
    }, [graphData, chargeStrength, linkDistance])

    const handleZoomIn = () => {
        if (fgRef.current) {
            const currentZoom = fgRef.current.zoom() as number
            fgRef.current.zoom(currentZoom, 400)
        }
    }

    const handleZoomOut = () => {
        if (fgRef.current) {
            const currentZoom = fgRef.current.zoom() as number
            fgRef.current.zoom(currentZoom, 400)
        }
    }

    const handleReset = () => {
        if (fgRef.current) {
            fgRef.current.zoomToFit(400, 20)
        }
    }

    return (
        <Card className="h-full flex flex-col overflow-hidden relative border-none shadow-none bg-background/50">
            <div className="absolute top-4 right-4 z-10 flex flex-col gap-2 bg-background/80 backdrop-blur p-2 rounded-lg border shadow-sm">
                <Button variant="ghost" size="icon" onClick={handleZoomIn} title="Zoom In">
                    <ZoomIn className="h-4 w-4" />
                </Button>
                <Button variant="ghost" size="icon" onClick={handleZoomOut} title="Zoom Out">
                    <ZoomOut className="h-4 w-4" />
                </Button>
                <Button variant="ghost" size="icon" onClick={handleReset} title="Reset View">
                    <RotateCcw className="h-4 w-4" />
                </Button>

                <Popover>
                    <PopoverTrigger asChild>
                        <Button variant="ghost" size="icon" title="Graph Settings">
                            <Settings2 className="h-4 w-4" />
                        </Button>
                    </PopoverTrigger>
                    <PopoverContent side="left" className="w-80 p-4 mr-2 bg-background/95 backdrop-blur">
                        <div className="space-y-4">
                            <div className="space-y-2">
                                <Label>Repulsion Force ({Math.abs(chargeStrength)})</Label>
                                <Slider
                                    value={[Math.abs(chargeStrength)]}
                                    min={50}
                                    max={1000}
                                    step={10}
                                    onValueChange={(vals) => setChargeStrength(-vals[0])}
                                />
                                <p className="text-xs text-muted-foreground">Higher values spread nodes further apart.</p>
                            </div>
                            <div className="space-y-2">
                                <Label>Link Distance ({linkDistance})</Label>
                                <Slider
                                    value={[linkDistance]}
                                    min={10}
                                    max={200}
                                    step={5}
                                    onValueChange={(vals) => setLinkDistance(vals[0])}
                                />
                            </div>
                        </div>
                    </PopoverContent>
                </Popover>
            </div>

            <div className="flex-1 w-full h-full min-h-[600px]" ref={containerRef}>
                <ForceGraph2D
                    ref={fgRef}
                    width={dimensions.width}
                    height={dimensions.height}
                    graphData={graphData}
                    nodeLabel={(node: any) => `${node.name}\nStatus: ${node.status}\nIn-links: ${node.inDegree}\nOut-links: ${node.outDegree}\nIssues: ${node.issueCount}`}
                    nodeRelSize={8}
                    linkColor={(link: any) => link.color}
                    linkWidth={1}
                    linkDirectionalArrowLength={6}
                    linkDirectionalArrowRelPos={1}
                    linkDirectionalArrowColor={() => theme === 'dark' ? 'rgba(255,255,255,0.5)' : 'rgba(0,0,0,0.5)'}
                    backgroundColor="rgba(0,0,0,0)"
                    nodeCanvasObjectMode={() => 'replace'}
                    nodeCanvasObject={(node: any, ctx: any, globalScale: any) => {
                        // Smaller radius base (User requested smaller circles)
                        const r = Math.max(2, new Number(node.val).valueOf());

                        // Draw shadow
                        ctx.shadowColor = node.color;
                        ctx.shadowBlur = 10;
                        ctx.beginPath();
                        ctx.arc(node.x, node.y, r, 0, 2 * Math.PI, false);
                        ctx.fillStyle = node.color;
                        ctx.fill();

                        // Reset shadow
                        ctx.shadowBlur = 0;

                        // Draw border

                        ctx.lineWidth = 1.5 / globalScale;
                        ctx.strokeStyle = theme === 'dark' ? '#ffffff' : '#000000';
                        ctx.stroke();

                        // Label optimization: Only draw if zoomed in or specialized nodes?
                        // For now we rely on tooltip, cleaner view requested.
                    }}
                    onNodeClick={(node: any) => {
                        if (onNodeClick) onNodeClick(node.id)
                        fgRef.current.centerAt(node.x, node.y, 1000);
                        fgRef.current.zoom(3, 2000);
                    }}
                    cooldownTicks={100}
                    d3AlphaDecay={0.02}
                    d3VelocityDecay={0.4}
                    warmupTicks={100}
                />
            </div>

            <div className="p-4 border-t flex gap-4 text-xs text-muted-foreground justify-center">
                <div className="flex items-center gap-2">
                    <span className="w-3 h-3 rounded-full bg-[oklch(0.65_0.18_145)]"></span> Healthy
                </div>
                <div className="flex items-center gap-2">
                    <span className="w-3 h-3 rounded-full bg-[oklch(0.75_0.15_85)]"></span> Warning
                </div>
                <div className="flex items-center gap-2">
                    <span className="w-3 h-3 rounded-full bg-[oklch(0.65_0.15_250)]"></span> Critical
                </div>
                <div className="flex items-center gap-2">
                    <span className="w-3 h-3 rounded-full bg-[oklch(0.55_0.2_25)]"></span> Error
                </div>
            </div>
        </Card >
    )
}
