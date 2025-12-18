"use client"

import { useMemo, useRef, useState, useEffect, useCallback } from "react"
import { useTheme } from "next-themes"
import type { CompleteAnalysisResult } from "@/src/lib/types"
import { Card } from "@/src/components/ui/card"
import { ZoomIn, ZoomOut, RotateCcw, Settings2, X, ExternalLink } from "lucide-react"
import { Button } from "@/src/components/ui/button"
import { Slider } from "@/src/components/ui/slider"
import { Popover, PopoverContent, PopoverTrigger } from "@/src/components/ui/popover"
import { Label } from "@/src/components/ui/label"

export interface GraphViewProps {
    data: CompleteAnalysisResult
    onNodeClick?: (url: string) => void
    onSelectPage?: (index: number) => void
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

interface GraphData {
    nodes: GraphNode[]
    links: GraphLink[]
}

interface NodeDegrees {
    inDegree: Map<string, number>
    outDegree: Map<string, number>
}

// Constants
const DEFAULT_REPULSION = 10
const DEFAULT_LINK_DISTANCE = 100
const MIN_GRAPH_HEIGHT = 600
const TOOLTIP_OFFSET = 15
const RESIZE_DEBOUNCE_DELAY = 100

const GRAPH_CONFIG = {
    simulation: {
        linkSpring: 0.3,
        friction: 0.1,
        decay: 1000,
    },
    simulationGravity: 0,
    simulationCenter: 1,
    renderLinks: true,
    linkWidth: 0.1,
    spaceSize: 8192,
    nodeSizeScale: 0.25,
    scalePointsOnZoom: false,
    scaleLinksOnZoom: false,
    curvedLinks: true,
    linkWidthScale: 0,
    linkOpacity: 0.3,
    linkDefaultArrows: true,
    linkArrowsSizeScale: 0.5,
    simulationLinkDistance: 100,
    pointSizeScale: 1,
    simulationFriction: 0,
    enableSimulationDuringZoom: true
}

const NODE_COLORS = {
    healthy: '#46c773ff',
    warning: '#e8aa3fff',
    critical: '#f14444ff',
    error: '#ff0000ff',
    dimmed: '#666666ff'
} as const

// Utility Functions
const normalizeUrl = (url: string): string => {
    try {
        const parsed = new URL(url)
        return (parsed.origin + parsed.pathname).replace(/\/$/, "")
    } catch {
        return url.replace(/\/$/, "")
    }
}

const resolveInternalUrl = (href: string, baseUrl: string, validUrls: Map<string, string>): string | null => {
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

const calculateNodeDegrees = (
    pages: CompleteAnalysisResult['pages'],
    validUrls: Map<string, string>
): NodeDegrees => {
    const inDegree = new Map<string, number>()
    const outDegree = new Map<string, number>()

    pages.forEach(page => {
        inDegree.set(page.url, 0)
        outDegree.set(page.url, 0)
    })

    pages.forEach(page => {
        if (!page.detailed_links) return

        let outgoingCount = 0

        page.detailed_links.forEach(link => {
            if (!link.is_internal) return

            const targetUrl = resolveInternalUrl(link.href, page.url, validUrls)
            if (targetUrl) {
                outgoingCount++
                inDegree.set(targetUrl, (inDegree.get(targetUrl) || 0) + 1)
            }
        })

        outDegree.set(page.url, outgoingCount)
    })

    return { inDegree, outDegree }
}

const getNodeColor = (issues: CompleteAnalysisResult['issues'], pageUrl: string): string => {
    const pageIssues = issues.filter(issue => issue.page_url === pageUrl)

    if (pageIssues.some(i => i.issue_type === "Critical")) return NODE_COLORS.critical
    if (pageIssues.some(i => i.issue_type === "Warning")) return NODE_COLORS.warning

    return NODE_COLORS.healthy
}

const calculateNodeSize = (inDegree: number): number => {
    return 2 + Math.log(inDegree + 1) * 2
}

// Custom Hooks
const useGraphData = (data: CompleteAnalysisResult, selectedNodeId: string | null): GraphData => {
    return useMemo(() => {
        const nodes: GraphNode[] = []
        const links: GraphLink[] = []

        const validUrls = new Map<string, string>()
        data.pages.forEach(page => {
            validUrls.set(normalizeUrl(page.url), page.url)
        })

        const { inDegree, outDegree } = calculateNodeDegrees(data.pages, validUrls)

        // Create all nodes
        data.pages.forEach(page => {
            const issuesForPage = data.issues.filter(i => i.page_url === page.url)
            const baseColor = getNodeColor(data.issues, page.url)

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

        // Create links
        data.pages.forEach(page => {
            if (!page.detailed_links) return

            page.detailed_links.forEach(link => {
                if (!link.is_internal) return

                const targetUrl = resolveInternalUrl(link.href, page.url, validUrls)
                if (!targetUrl) return

                const targetPage = data.pages.find(p => p.url === targetUrl)
                const isBroken = targetPage?.status_code ? targetPage.status_code >= 400 : false

                links.push({
                    source: page.url,
                    target: targetUrl,
                    isBroken
                })
            })
        })

        // Filter links if a node is selected
        if (selectedNodeId) {
            const filteredLinks = links.filter(
                link => link.source === selectedNodeId || link.target === selectedNodeId
            )

            // Get connected node IDs
            const connectedNodeIds = new Set([selectedNodeId])
            filteredLinks.forEach(link => {
                connectedNodeIds.add(link.source)
                connectedNodeIds.add(link.target)
            })

            // Dim nodes that aren't connected
            nodes.forEach(node => {
                if (!connectedNodeIds.has(node.id)) {
                    node.color = NODE_COLORS.dimmed
                }
            })

            return { nodes, links: filteredLinks }
        }

        return { nodes, links }
    }, [data, selectedNodeId])
}

const useContainerDimensions = (containerRef: React.RefObject<HTMLDivElement | null>) => {
    const [dimensions, setDimensions] = useState({ width: 800, height: 600 })

    useEffect(() => {
        const updateDimensions = () => {
            if (containerRef.current) {
                setDimensions({
                    width: containerRef.current.clientWidth,
                    height: containerRef.current.clientHeight
                })
            }
        }

        const timeoutId = setTimeout(updateDimensions, RESIZE_DEBOUNCE_DELAY)
        updateDimensions()

        window.addEventListener('resize', updateDimensions)
        return () => {
            clearTimeout(timeoutId)
            window.removeEventListener('resize', updateDimensions)
        }
    }, [containerRef])

    return dimensions
}

// Main Component
export function GraphView({ data, onNodeClick, onSelectPage }: GraphViewProps) {
    const { resolvedTheme } = useTheme()
    const theme = resolvedTheme || 'dark'

    const canvasRef = useRef<HTMLCanvasElement>(null)
    const containerRef = useRef<HTMLDivElement>(null)
    const cosmographRef = useRef<any>(null)

    const [repulsion, setRepulsion] = useState(DEFAULT_REPULSION)
    const [linkDistance, setLinkDistance] = useState(DEFAULT_LINK_DISTANCE)
    const [hoveredNode, setHoveredNode] = useState<GraphNode | null>(null)
    const [selectedNode, setSelectedNode] = useState<GraphNode | null>(null)
    const [mousePos, setMousePos] = useState({ x: 0, y: 0 })
    const [isLoading, setIsLoading] = useState(true)

    const dimensions = useContainerDimensions(containerRef)
    const { nodes, links } = useGraphData(data, selectedNode?.id || null)

    const handleNodeClick = useCallback((node?: GraphNode) => {
        console.log("CLICK")
        if (node) {
            setSelectedNode(node)
            if (onNodeClick) {
                onNodeClick(node.url)
            }
        }
    }, [onNodeClick])

    const handleClearSelection = useCallback(() => {
        setSelectedNode(null)
    }, [])

    const handleViewPageDetails = useCallback(() => {
        if (selectedNode && onSelectPage) {
            const pageIndex = data.pages.findIndex(p => p.url === selectedNode.url)
            if (pageIndex !== -1) {
                onSelectPage(pageIndex)
            }
        }
    }, [selectedNode, data.pages, onSelectPage])

    const handleNodeMouseOver = useCallback((node?: GraphNode) => {
        if (node) setHoveredNode(node)
    }, [])

    const handleNodeMouseOut = useCallback(() => {
        setHoveredNode(null)
    }, [])

    // Initialize Cosmograph
    useEffect(() => {
        let mounted = true

        const initCosmograph = async () => {
            if (!canvasRef.current || nodes.length === 0) return

            try {
                const { Graph } = await import('@cosmograph/cosmos')
                if (!mounted) return

                const config = {
                    ...GRAPH_CONFIG,
                    simulation: {
                        ...GRAPH_CONFIG.simulation,
                        repulsion,
                        linkDistance
                    },
                    nodeSize: (node: GraphNode) => calculateNodeSize(node.inDegree),
                    nodeColor: (node: GraphNode) => node.color,
                    linkColor: () => "#d5d2d2ff",
                    backgroundColor: theme === 'dark' ? '#000000' : '#ffffff',
                    onClick: handleNodeClick,
                    onNodeMouseOver: handleNodeMouseOver,
                    onNodeMouseOut: handleNodeMouseOut
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
            cosmographRef.current?.destroy?.()
            cosmographRef.current = null
        }
    }, [nodes, links, theme, repulsion, linkDistance, handleNodeClick, handleNodeMouseOver, handleNodeMouseOut])

    // Update canvas dimensions
    useEffect(() => {
        if (cosmographRef.current && canvasRef.current) {
            canvasRef.current.width = dimensions.width
            canvasRef.current.height = dimensions.height
            cosmographRef.current.fitView()
        }
    }, [dimensions])

    const handleZoomIn = useCallback(() => {
        cosmographRef.current?.zoomIn()
    }, [])

    const handleZoomOut = useCallback(() => {
        cosmographRef.current?.zoomOut()
    }, [])

    const handleReset = useCallback(() => {
        cosmographRef.current?.fitView()
    }, [])

    const handleMouseMove = useCallback((e: React.MouseEvent) => {
        if (hoveredNode) {
            setMousePos({ x: e.clientX, y: e.clientY })
        }
    }, [hoveredNode])

    const handleRepulsionChange = useCallback((values: number[]) => {
        setRepulsion(values[0] / 100)
    }, [])

    const handleLinkDistanceChange = useCallback((values: number[]) => {
        setLinkDistance(values[0] / 10)
    }, [])

    return (
        <Card className="h-full flex flex-col overflow-hidden relative border-none shadow-none bg-background/50">
            <GraphControls
                onZoomIn={handleZoomIn}
                onZoomOut={handleZoomOut}
                onReset={handleReset}
                repulsion={repulsion}
                linkDistance={linkDistance}
                onRepulsionChange={handleRepulsionChange}
                onLinkDistanceChange={handleLinkDistanceChange}
            />

            {selectedNode && (
                <SelectedNodePanel
                    node={selectedNode}
                    onClear={handleClearSelection}
                    onViewDetails={handleViewPageDetails}
                />
            )}

            {hoveredNode && !selectedNode && (
                <NodeTooltip node={hoveredNode} position={mousePos} />
            )}

            <div
                className="flex-1 w-full h-full min-h-[600px] relative"
                ref={containerRef}
                onMouseMove={handleMouseMove}
            >
                {isLoading && <LoadingOverlay />}
                <canvas
                    ref={canvasRef}
                    width={dimensions.width}
                    height={dimensions.height}
                    className="w-full h-full"
                    style={{ display: 'block' }}
                />
            </div>

            <GraphLegend />
        </Card>
    )
}

// Sub-components
interface GraphControlsProps {
    onZoomIn: () => void
    onZoomOut: () => void
    onReset: () => void
    repulsion: number
    linkDistance: number
    onRepulsionChange: (values: number[]) => void
    onLinkDistanceChange: (values: number[]) => void
}

function GraphControls({
    onZoomIn,
    onZoomOut,
    onReset,
    repulsion,
    linkDistance,
    onRepulsionChange,
    onLinkDistanceChange
}: GraphControlsProps) {
    return (
        <div className="absolute top-4 right-4 z-10 flex flex-col gap-2 bg-background/80 backdrop-blur p-2 rounded-lg border shadow-sm">
            <Button variant="ghost" size="icon" onClick={onZoomIn} title="Zoom In">
                <ZoomIn className="h-4 w-4" />
            </Button>
            <Button variant="ghost" size="icon" onClick={onZoomOut} title="Zoom Out">
                <ZoomOut className="h-4 w-4" />
            </Button>
            <Button variant="ghost" size="icon" onClick={onReset} title="Reset View">
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
                                onValueChange={onRepulsionChange}
                            />
                            <p className="text-xs text-muted-foreground">
                                Higher values spread nodes further apart.
                            </p>
                        </div>
                        <div className="space-y-2">
                            <Label>Link Distance ({linkDistance.toFixed(1)})</Label>
                            <Slider
                                value={[linkDistance * 10]}
                                min={5}
                                max={5000}
                                step={1}
                            />
                        </div>
                    </div>
                </PopoverContent>
            </Popover>
        </div>
    )
}

interface SelectedNodePanelProps {
    node: GraphNode
    onClear: () => void
    onViewDetails: () => void
}

function SelectedNodePanel({ node, onClear, onViewDetails }: SelectedNodePanelProps) {
    return (
        <div className="absolute top-4 left-4 z-10 bg-background/95 backdrop-blur border rounded-lg shadow-lg p-4 max-w-md">
            <div className="flex items-start justify-between gap-3 mb-3">
                <div className="flex-1 min-w-0">
                    <h3 className="font-semibold text-sm truncate">{node.title}</h3>
                    <p className="text-xs text-muted-foreground truncate">{node.url}</p>
                </div>
                <Button
                    variant="ghost"
                    size="icon"
                    className="h-6 w-6 shrink-0"
                    onClick={onClear}
                    title="Clear Selection"
                >
                    <X className="h-4 w-4" />
                </Button>
            </div>

            <div className="grid grid-cols-2 gap-3 text-xs mb-3">
                <div>
                    <div className="text-muted-foreground">Status</div>
                    <div className="font-medium">{node.status || 'N/A'}</div>
                </div>
                <div>
                    <div className="text-muted-foreground">Issues</div>
                    <div className="font-medium">{node.issueCount}</div>
                </div>
                <div>
                    <div className="text-muted-foreground">Incoming</div>
                    <div className="font-medium">{node.inDegree} links</div>
                </div>
                <div>
                    <div className="text-muted-foreground">Outgoing</div>
                    <div className="font-medium">{node.outDegree} links</div>
                </div>
            </div>

            <Button
                onClick={onViewDetails}
                className="w-full"
                size="sm"
            >
                <ExternalLink className="h-3 w-3 mr-2" />
                View Page Details
            </Button>
        </div>
    )
}

interface NodeTooltipProps {
    node: GraphNode
    position: { x: number; y: number }
}

function NodeTooltip({ node, position }: NodeTooltipProps) {
    return (
        <div
            className="absolute z-20 pointer-events-none bg-background/95 backdrop-blur border rounded-lg shadow-lg p-3 text-sm max-w-xs"
            style={{
                left: position.x + TOOLTIP_OFFSET,
                top: position.y + TOOLTIP_OFFSET,
            }}
        >
            <div className="font-medium truncate">{node.title}</div>
            <div className="text-xs text-muted-foreground truncate">{node.url}</div>
            <div className="mt-2 space-y-1 text-xs">
                <div>Status: {node.status || 'N/A'}</div>
                <div>In-links: {node.inDegree}</div>
                <div>Out-links: {node.outDegree}</div>
                <div>Issues: {node.issueCount}</div>
            </div>
        </div>
    )
}

function LoadingOverlay() {
    return (
        <div className="absolute inset-0 flex items-center justify-center bg-muted/10 z-10">
            <div className="flex flex-col items-center gap-3">
                <div className="w-8 h-8 border-4 border-primary border-t-transparent rounded-full animate-spin" />
                <div className="text-sm text-muted-foreground">Loading Graph...</div>
            </div>
        </div>
    )
}

function GraphLegend() {
    return (
        <div className="p-4 border-t flex gap-4 text-xs text-muted-foreground justify-center">
            <LegendItem color={NODE_COLORS.healthy} label="Healthy" />
            <LegendItem color={NODE_COLORS.warning} label="Warning" />
            <LegendItem color={NODE_COLORS.critical} label="Critical" />
            <LegendItem color={NODE_COLORS.error} label="Error" />
        </div>
    )
}

interface LegendItemProps {
    color: string
    label: string
}

function LegendItem({ color, label }: LegendItemProps) {
    return (
        <div className="flex items-center gap-2">
            <div
                className="w-3 h-3 rounded-full"
                style={{ backgroundColor: color }}
            />
            {label}
        </div>
    )
}