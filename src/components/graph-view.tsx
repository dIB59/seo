"use client"

import { useMemo, useRef, useState, useEffect } from "react"
import { useTheme } from "next-themes"
import type { CompleteAnalysisResult } from "@/src/lib/types"
import { Card } from "@/src/components/ui/card"
import { ZoomIn, ZoomOut, RotateCcw, Settings2 } from "lucide-react"
import { Button } from "@/src/components/ui/button"
import { Slider } from "@/src/components/ui/slider"
import { Popover, PopoverContent, PopoverTrigger } from "@/src/components/ui/popover"
import { Label } from "@/src/components/ui/label"

interface GraphViewProps {
    data: CompleteAnalysisResult
    onNodeClick?: (url: string) => void
}

interface GraphNode {
    id: string
    url: string
    title: string
    status: number | null
    issueCount: number
    inDegree: number
    outDegree: number
    color: string
}

interface GraphLink {
    source: string
    target: string
    isBroken: boolean
}

export function GraphView({ data, onNodeClick }: GraphViewProps) {
    const { resolvedTheme } = useTheme()
    const theme = resolvedTheme || 'dark'

    const canvasRef = useRef<HTMLCanvasElement>(null)
    const containerRef = useRef<HTMLDivElement>(null)
    const cosmographRef = useRef<any>(null)
    const [dimensions, setDimensions] = useState({ width: 800, height: 600 })
    const [repulsion, setRepulsion] = useState(0.8)
    const [linkDistance, setLinkDistance] = useState(20)
    const [hoveredNode, setHoveredNode] = useState<GraphNode | null>(null)
    const [mousePos, setMousePos] = useState({ x: 0, y: 0 })
    const [isLoading, setIsLoading] = useState(true)

    // Resize handler
    useEffect(() => {
        const updateDimensions = () => {
            if (containerRef.current) {
                const width = containerRef.current.clientWidth
                const height = containerRef.current.clientHeight
                setDimensions({ width, height })
            }
        }

        window.addEventListener('resize', updateDimensions)
        updateDimensions()
        setTimeout(updateDimensions, 100)

        return () => window.removeEventListener('resize', updateDimensions)
    }, [])

    const { nodes, links } = useMemo(() => {
        const nodes: GraphNode[] = []
        const links: GraphLink[] = []
        const inDegree = new Map<string, number>()
        const outDegree = new Map<string, number>()

        const normalizeUrl = (url: string) => {
            try {
                const u = new URL(url)
                return (u.origin + u.pathname).replace(/\/$/, "")
            } catch {
                return url.replace(/\/$/, "")
            }
        }

        const validNormalizedUrls = new Map<string, string>()

        data.pages.forEach(page => {
            const normalized = normalizeUrl(page.url)
            validNormalizedUrls.set(normalized, page.url)
            inDegree.set(page.url, 0)
            outDegree.set(page.url, 0)
        })

        data.pages.forEach(page => {
            if (page.detailed_links) {
                let currentOut = 0
                page.detailed_links.forEach(link => {
                    if (link.is_internal) {
                        const targetNormalized = normalizeUrl(link.href)
                        let targetOriginalUrl = validNormalizedUrls.get(targetNormalized)

                        if (!targetOriginalUrl && !link.href.startsWith('http')) {
                            try {
                                const baseUrl = new URL(page.url)
                                const absoluteUrl = new URL(link.href, baseUrl.origin).href
                                targetOriginalUrl = validNormalizedUrls.get(normalizeUrl(absoluteUrl))
                            } catch (e) {
                                // ignore
                            }
                        }

                        if (targetOriginalUrl) {
                            currentOut++
                            inDegree.set(targetOriginalUrl, (inDegree.get(targetOriginalUrl) || 0) + 1)
                        }
                    }
                })
                outDegree.set(page.url, currentOut)
            }
        })

        data.pages.forEach((page) => {
            const issuesForPage = data.issues.filter(i => i.page_url === page.url)
            const isCritical = issuesForPage.some(i => i.issue_type === "Critical")
            const isWarning = issuesForPage.some(i => i.issue_type === "Warning")


            const node_color = "#b3b3b3"

            const incoming = inDegree.get(page.url) || 0
            const outgoing = outDegree.get(page.url) || 0

            nodes.push({
                id: page.url,
                url: page.url,
                title: page.title || "No Title",
                status: page.status_code,
                issueCount: issuesForPage.length,
                inDegree: incoming,
                outDegree: outgoing,
                color: node_color
            })
        })

        data.pages.forEach((page) => {
            if (page.detailed_links) {
                page.detailed_links.forEach(link => {
                    if (link.is_internal) {
                        const targetNormalized = normalizeUrl(link.href)
                        let targetOriginalUrl = validNormalizedUrls.get(targetNormalized)

                        if (!targetOriginalUrl && !link.href.startsWith('http')) {
                            try {
                                const baseUrl = new URL(page.url)
                                const absoluteUrl = new URL(link.href, baseUrl.origin).href
                                targetOriginalUrl = validNormalizedUrls.get(normalizeUrl(absoluteUrl))
                            } catch (e) {
                                // ignore
                            }
                        }

                        if (targetOriginalUrl) {
                            const targetPage = data.pages.find(p => p.url === targetOriginalUrl)
                            const isBroken = targetPage && targetPage.status_code && targetPage.status_code >= 400

                            links.push({
                                source: page.url,
                                target: targetOriginalUrl,
                                isBroken: !!isBroken
                            })
                        }
                    }
                })
            }
        })

        return { nodes, links }
    }, [data])

    // Initialize Cosmograph
    useEffect(() => {
        let mounted = true

        const initCosmograph = async () => {
            if (!canvasRef.current || nodes.length === 0) return

            try {
                const { Graph } = await import('@cosmograph/cosmos')

                if (!mounted) return

                const config = {
                    simulation: {
                        repulsion: repulsion,
                        linkDistance: linkDistance,
                        linkSpring: 0.5,
                        friction: 0.85,
                        gravity: 0.1,
                        center: 1.0,
                        decay: 5000
                    },
                    renderLinks: true,
                    linkArrows: true,
                    linkWidth: 1,
                    nodeSize: (node: GraphNode) => 2 + Math.log(node.inDegree + 1) * 2,
                    nodeColor: (node: GraphNode) => node.color,
                    linkColor: (link: GraphLink) => "#ffffff",
                    backgroundColor: theme === 'dark' ? '#000000' : '#ffffff',
                    spaceSize: 8192,
                    onClick: (node?: GraphNode) => {
                        if (node && onNodeClick) {
                            onNodeClick(node.url)
                        }
                    },
                    onNodeMouseOver: (node?: GraphNode) => {
                        if (node) setHoveredNode(node)
                    },
                    onNodeMouseOut: () => {
                        setHoveredNode(null)
                    }
                }

                const graph = new Graph(canvasRef.current, config)
                cosmographRef.current = graph

                graph.setData(nodes, links)
                graph.fitView()

                setIsLoading(false)
            } catch (error) {
                console.error('Failed to initialize Cosmograph:', error)
                setIsLoading(false)
            }
        }

        initCosmograph()

        return () => {
            mounted = false
            if (cosmographRef.current) {
                cosmographRef.current.destroy?.()
                cosmographRef.current = null
            }
        }
    }, [nodes, links, theme, repulsion, linkDistance, onNodeClick])

    // Update dimensions
    useEffect(() => {
        if (cosmographRef.current && canvasRef.current) {
            canvasRef.current.width = dimensions.width
            canvasRef.current.height = dimensions.height
            cosmographRef.current.fitView()
        }
    }, [dimensions])

    const handleZoomIn = () => {
        cosmographRef.current?.zoomIn()
    }

    const handleZoomOut = () => {
        cosmographRef.current?.zoomOut()
    }

    const handleReset = () => {
        cosmographRef.current?.fitView()
    }

    const handleMouseMove = (e: React.MouseEvent) => {
        if (hoveredNode) {
            setMousePos({ x: e.clientX, y: e.clientY })
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
                                <Label>Repulsion Force ({repulsion.toFixed(2)})</Label>
                                <Slider
                                    value={[repulsion * 100]}
                                    min={10}
                                    max={2000}
                                    step={5}
                                    onValueChange={(vals) => setRepulsion(vals[0] / 100)}
                                />
                                <p className="text-xs text-muted-foreground">Higher values spread nodes further apart.</p>
                            </div>
                            <div className="space-y-2">
                                <Label>Link Distance ({linkDistance.toFixed(1)})</Label>
                                <Slider
                                    value={[linkDistance * 10]}
                                    min={5}
                                    max={500}
                                    step={1}
                                    onValueChange={(vals) => setLinkDistance(vals[0] / 10)}
                                />
                            </div>
                        </div>
                    </PopoverContent>
                </Popover>
            </div>

            {hoveredNode && (
                <div
                    className="absolute z-20 pointer-events-none bg-background/95 backdrop-blur border rounded-lg shadow-lg p-3 text-sm max-w-xs"
                    style={{
                        left: mousePos.x + 15,
                        top: mousePos.y + 15,
                    }}
                >
                    <div className="font-medium truncate">{hoveredNode.title}</div>
                    <div className="text-xs text-muted-foreground truncate">{hoveredNode.url}</div>
                    <div className="mt-2 space-y-1 text-xs">
                        <div>Status: {hoveredNode.status || 'N/A'}</div>
                        <div>In-links: {hoveredNode.inDegree}</div>
                        <div>Out-links: {hoveredNode.outDegree}</div>
                        <div>Issues: {hoveredNode.issueCount}</div>
                    </div>
                </div>
            )}

            <div
                className="flex-1 w-full h-full min-h-[600px] relative"
                ref={containerRef}
                onMouseMove={handleMouseMove}
            >
                {isLoading && (
                    <div className="absolute inset-0 flex items-center justify-center bg-muted/10 z-10">
                        <div className="flex flex-col items-center gap-3">
                            <div className="w-8 h-8 border-4 border-primary border-t-transparent rounded-full animate-spin"></div>
                            <div className="text-sm text-muted-foreground">Loading Graph...</div>
                        </div>
                    </div>
                )}
                <canvas
                    ref={canvasRef}
                    width={dimensions.width}
                    height={dimensions.height}
                    className="w-full h-full"
                    style={{ display: 'block' }}
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
        </Card>
    )
}