import { ZoomIn, ZoomOut, RotateCcw, Settings2 } from "lucide-react"
import { Button } from "@/src/components/ui/button"
import { Popover, PopoverContent, PopoverTrigger } from "@/src/components/ui/popover"
import { Slider } from "@/src/components/ui/slider"
import { Label } from "@/src/components/ui/label"

interface GraphControlsProps {
    onZoomIn: () => void
    onZoomOut: () => void
    onReset: () => void
    repulsion: number
    linkDistance: number
    onRepulsionChange: (values: number[]) => void
    onLinkDistanceChange: (values: number[]) => void
}

export default function GraphControls({ onZoomIn, onZoomOut, onReset, repulsion, linkDistance, onRepulsionChange, onLinkDistanceChange }: GraphControlsProps) {
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
                            <p className="text-xs text-muted-foreground">Higher values spread nodes further apart.</p>
                        </div>
                        <div className="space-y-2">
                            <Label>Link Distance ({linkDistance.toFixed(1)})</Label>
                            <Slider value={[linkDistance * 10]} min={5} max={5000} step={1} onValueChange={onLinkDistanceChange} />
                        </div>
                    </div>
                </PopoverContent>
            </Popover>
        </div>
    )
}
