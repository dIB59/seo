export default function NodeTooltip({ node, position }: { node: any, position: { x: number, y: number } }) {
    const TOOLTIP_OFFSET = 15
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
