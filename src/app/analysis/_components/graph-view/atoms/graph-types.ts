/** Typed graph node for the Cosmograph force-directed graph. */
export interface GraphNode {
    id: string
    url: string
    title: string
    status: number | null
    issueCount: number
    inDegree: number
    outDegree: number
    color: string
}

/** Typed link between two graph nodes. */
export interface GraphLink {
    source: string
    target: string
    isBroken: boolean
}

/** Subset of the Cosmograph Graph API we interact with. */
export interface CosmographInstance {
    setData: (nodes: GraphNode[], links: GraphLink[]) => void
    fitView: () => void
    zoomIn: () => void
    zoomOut: () => void
    destroy: () => void
}
