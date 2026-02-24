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
        <div className="relative group">
            {/* Control Bar Container */}
            <div className="flex items-center p-1 ml-1 bg-card border border-border rounded-lg shadow-sm focus-within:ring-2 focus-within:ring-primary/20 focus-within:border-primary/50 transition-all duration-300">
                {/* Icon Marker */}
                <div className="flex items-center justify-center w-10 h-10 text-muted-foreground/50 border-r border-border/40">
                    <Globe className="h-4 w-4" />
                </div>

                {/* Input Field */}
                <div className="relative flex-1">
                    <Input
                        type="text"
                        placeholder="https://example.com"
                        value={url}
                        onChange={(e) => setUrl(e.target.value)}
                        className="w-full h-10 px-4 bg-transparent border-none shadow-none focus-visible:ring-0 text-base font-mono placeholder:font-sans placeholder:text-muted-foreground/40"
                        required
                    />
                    {url && (
                        <button
                            type="button"
                            onClick={onClear}
                            className="absolute right-2 top-1/2 -translate-y-1/2 p-1.5 text-muted-foreground/40 hover:text-foreground hover:bg-muted/50 rounded-md transition-all"
                        >
                            <X className="h-3.5 w-3.5" />
                        </button>
                    )}
                </div>

                {/* Action Button */}
                <Button
                    type="submit"
                    disabled={isLoading || !isValid}
                    className="h-9 px-6 m-0.5 rounded-lg font-medium shadow-none transition-all active:scale-95"
                >
                    {isLoading ? (
                        <div className="flex items-center gap-2">
                            <span className="w-2 h-2 rounded-full bg-background animate-pulse" />
                            <span className="opacity-80">Running...</span>
                        </div>
                    ) : (
                        <div className="flex items-center gap-2">
                            <Play className="h-3.5 w-3.5 fill-current" />
                            <span>Analyze</span>
                        </div>
                    )}
                </Button>
            </div>
        </div>
    )
}
