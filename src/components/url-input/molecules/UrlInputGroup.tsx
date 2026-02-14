import { Globe, Play, X } from "lucide-react"
import { Button } from "@/src/components/ui/button"
import { Input } from "@/src/components/ui/input"

interface UrlInputGroupProps {
    url: string
    setUrl: (url: string) => void
    onClear: () => void
    isLoading: boolean
    isValid: boolean
}

export function UrlInputGroup({ url, setUrl, onClear, isLoading, isValid }: UrlInputGroupProps) {
    return (
        <div className="flex gap-3">
            <div className="relative flex-1">
                <Globe className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                <Input
                    type="text"
                    placeholder="Enter website URL to analyze (e.g., https://example.com)"
                    value={url}
                    onChange={(e) => setUrl(e.target.value)}
                    className="pl-10 pr-10 bg-secondary border-border h-11"
                    required
                />
                {url && (
                    <button
                        type="button"
                        onClick={onClear}
                        className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors"
                    >
                        <X className="h-4 w-4" />
                    </button>
                )}
            </div>
            <Button type="submit" disabled={isLoading || !isValid} className="h-11 px-6">
                <Play className="h-4 w-4 mr-2" />
                {isLoading ? "Starting..." : "Analyze"}
            </Button>
        </div>
    )
}
