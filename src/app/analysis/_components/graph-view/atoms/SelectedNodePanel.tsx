import { Button } from "@/src/components/ui/button"
import { X, ExternalLink } from "lucide-react"
import type { GraphNode } from "./graph-types"

interface SelectedNodePanelProps {
    node: GraphNode
    onClear: () => void
    onViewDetails: () => void
}

export default function SelectedNodePanel({ node, onClear, onViewDetails }: SelectedNodePanelProps) {
    return (
        <div className="absolute top-4 left-4 z-10 bg-background/95 backdrop-blur border border-border/60 rounded-lg shadow-lg overflow-hidden max-w-md animate-in slide-in-from-left-4 fade-in duration-300">
            {/* Colored health bar */}
            <div className="h-1 w-full" style={{ backgroundColor: node.color }} />

            <div className="p-4">
                <div className="flex items-start justify-between gap-3 mb-3">
                    <div className="flex-1 min-w-0">
                        <h3 className="font-semibold text-sm truncate">{node.title}</h3>
                        <p className="text-xs text-muted-foreground truncate">{node.url}</p>
                    </div>
                    <Button variant="ghost" size="icon" className="h-6 w-6 shrink-0" onClick={onClear} title="Clear Selection">
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

                <Button onClick={onViewDetails} className="w-full" size="sm">
                    <ExternalLink className="h-3 w-3 mr-2" />
                    View Page Details
                </Button>
            </div>
        </div>
    )
}
