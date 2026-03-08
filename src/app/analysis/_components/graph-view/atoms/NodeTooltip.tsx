import type { GraphNode } from "./graph-types"

export default function NodeTooltip({ node, position }: { node: GraphNode, position: { x: number, y: number } }) {
    const TOOLTIP_OFFSET = 15
    return (
        <div
            className="absolute z-20 pointer-events-none bg-background/95 backdrop-blur border border-border/60 rounded-lg shadow-lg p-3 text-sm max-w-xs animate-in fade-in duration-150"
            style={{
                left: position.x + TOOLTIP_OFFSET,
                top: position.y + TOOLTIP_OFFSET,
            }}
        >
            <div className="flex items-center gap-2">
                <span className="w-2 h-2 rounded-full shrink-0" style={{ backgroundColor: node.color }} />
                <span className="font-medium truncate">{node.title}</span>
            </div>
            <div className="text-xs text-muted-foreground truncate mt-0.5">{node.url}</div>
            <div className="mt-2 grid grid-cols-2 gap-x-4 gap-y-1 text-xs">
                <div className="text-muted-foreground">Status</div>
                <div className="font-medium text-right">{node.status || 'N/A'}</div>
                <div className="text-muted-foreground">In-links</div>
                <div className="font-medium text-right">{node.inDegree}</div>
                <div className="text-muted-foreground">Out-links</div>
                <div className="font-medium text-right">{node.outDegree}</div>
                <div className="text-muted-foreground">Issues</div>
                <div className="font-medium text-right">{node.issueCount}</div>
            </div>
        </div>
    )
}
