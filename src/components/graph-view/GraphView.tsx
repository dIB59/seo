"use client"

import { useRef, useState, useEffect, useCallback } from "react"
import { useTheme } from "next-themes"
import type { CompleteAnalysisResult } from "@/src/lib/types"
import GraphControls from "./molecules/GraphControls"
import SelectedNodePanel from "./atoms/SelectedNodePanel"
import NodeTooltip from "./atoms/NodeTooltip"
import LoadingOverlay from "./atoms/LoadingOverlay"
import GraphLegend from "./atoms/GraphLegend"
import { useGraphData, useContainerDimensions } from "./atoms/hooks"

interface GraphViewProps {
    data: CompleteAnalysisResult
    onNodeClick?: (url: string) => void
    onSelectPage?: (index: number) => void
}

export default function GraphView({ data, onNodeClick, onSelectPage }: GraphViewProps) {
    const { resolvedTheme } = useTheme()
    const theme = resolvedTheme || 'dark'

    const canvasRef = useRef<HTMLCanvasElement>(null)
    const containerRef = useRef<HTMLDivElement>(null)
    const cosmographRef = useRef<any>(null)

    const [repulsion, setRepulsion] = useState(10)
    const [linkDistance, setLinkDistance] = useState(100)
    const [hoveredNode, setHoveredNode] = useState<any | null>(null)
    const [selectedNode, setSelectedNode] = useState<any | null>(null)
    const [mousePos, setMousePos] = useState({ x: 0, y: 0 })
    const [isLoading, setIsLoading] = useState(true)

    const dimensions = useContainerDimensions(containerRef)
    const { nodes, links } = useGraphData(data, selectedNode?.id || null)

    const handleNodeClick = useCallback((node?: any) => {
        if (node) {
            setSelectedNode(node)
            if (onNodeClick) onNodeClick(node.url)
        }
    }, [onNodeClick])

    const handleClearSelection = useCallback(() => setSelectedNode(null), [])

    const handleViewPageDetails = useCallback(() => {
        if (selectedNode && onSelectPage) {
            const pageIndex = data.pages.findIndex(p => p.url === selectedNode.url)
            if (pageIndex !== -1) onSelectPage(pageIndex)
        }
    }, [selectedNode, data.pages, onSelectPage])

    const handleNodeMouseOver = useCallback((node?: any) => { if (node) setHoveredNode(node) }, [])
    const handleNodeMouseOut = useCallback(() => setHoveredNode(null), [])

    useEffect(() => {
        let mounted = true
        const init = async () => {
            if (!canvasRef.current || nodes.length === 0) return

            try {
                const { Graph } = await import('@cosmograph/cosmos')
                if (!mounted) return

                const config = {
                    simulation: { repulsion, linkDistance },
                    nodeSize: (node: any) => 2 + Math.log((node.inDegree || 0) + 1) * 2,
                    nodeColor: (node: any) => node.color,
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
            } catch (err) {
                console.error('Failed to init Cosmograph', err)
                setIsLoading(false)
            }
        }

        init()
        return () => { mounted = false; cosmographRef.current?.destroy?.(); cosmographRef.current = null }
    }, [nodes, links, theme, repulsion, linkDistance, handleNodeClick, handleNodeMouseOver, handleNodeMouseOut])

    useEffect(() => {
        if (cosmographRef.current && canvasRef.current) {
            canvasRef.current.width = dimensions.width
            canvasRef.current.height = dimensions.height
            cosmographRef.current.fitView()
        }
    }, [dimensions])

    const handleZoomIn = useCallback(() => cosmographRef.current?.zoomIn(), [])
    const handleZoomOut = useCallback(() => cosmographRef.current?.zoomOut(), [])
    const handleReset = useCallback(() => cosmographRef.current?.fitView(), [])

    const handleMouseMove = useCallback((e: any) => {
        if (hoveredNode) setMousePos({ x: e.clientX, y: e.clientY })
    }, [hoveredNode])

    return (
        <div className="h-[90vh] border-1">
            <div className="h-full flex flex-col overflow-hidden relative border-none shadow-none bg-background/50">
                <GraphControls
                    onZoomIn={handleZoomIn}
                    onZoomOut={handleZoomOut}
                    onReset={handleReset}
                    repulsion={repulsion}
                    linkDistance={linkDistance}
                    onRepulsionChange={(v) => setRepulsion(v[0] / 100)}
                    onLinkDistanceChange={(v) => setLinkDistance(v[0] / 10)}
                />

                {selectedNode && (
                    <SelectedNodePanel node={selectedNode} onClear={handleClearSelection} onViewDetails={handleViewPageDetails} />
                )}

                {hoveredNode && !selectedNode && (
                    <NodeTooltip node={hoveredNode} position={mousePos} />
                )}

                <div className="flex-1 w-full h-full min-h-[600px] relative" ref={containerRef} onMouseMove={handleMouseMove}>
                    {isLoading && <LoadingOverlay />}
                    <canvas ref={canvasRef} width={dimensions.width} height={dimensions.height} className="w-full h-full" style={{ display: 'block' }} />
                </div>

                <GraphLegend />
            </div>
        </div>
    )
}
