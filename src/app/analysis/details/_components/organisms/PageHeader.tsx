import { Popover, PopoverContent, PopoverTrigger } from "@/src/components/ui/popover"
import { Command, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList } from "@/src/components/ui/command"
import { Badge } from "@/src/components/ui/badge"
import { ChevronsUpDown, ChevronLeft, ChevronRight, Search } from "lucide-react"
import { Button } from "@/src/components/ui/button"
import { useState, useCallback } from "react"
import type { PageDetailData } from "@/src/lib/types"

export default function PageHeader({ page, pages, currentIndex, onBack, onNavigate }: {
    page: PageDetailData
    pages: PageDetailData[]
    currentIndex: number
    onBack: () => void
    onNavigate: (index: number) => void
}) {
    const [searchOpen, setSearchOpen] = useState(false)

    const canGoPrev = currentIndex > 0
    const canGoNext = currentIndex < pages.length - 1

    const goToPrev = useCallback(() => {
        if (canGoPrev) onNavigate(currentIndex - 1)
    }, [canGoPrev, currentIndex, onNavigate])

    const goToNext = useCallback(() => {
        if (canGoNext) onNavigate(currentIndex + 1)
    }, [canGoNext, currentIndex, onNavigate])

    const getShortPath = (url: string) => {
        try {
            return new URL(url).pathname || "/"
        } catch {
            return url.replace(/^https?:\/\/[^/]+/, "") || "/"
        }
    }

    return (
        <div className="flex items-center justify-between gap-4">
            <Button variant="ghost" size="sm" onClick={onBack}>
                <ChevronLeft className="h-4 w-4 mr-2" />
                Back to Results
            </Button>

            <div className="flex items-center gap-2 flex-1 justify-center max-w-md">
                <Popover open={searchOpen} onOpenChange={setSearchOpen}>
                    <PopoverTrigger asChild>
                        <Button
                            variant="outline"
                            role="combobox"
                            aria-expanded={searchOpen}
                            className="w-full justify-between text-left font-normal"
                        >
                            <div className="flex items-center gap-2 truncate">
                                <Search className="h-4 w-4 text-muted-foreground shrink-0" />
                                <span className="truncate">{getShortPath(page.url)}</span>
                            </div>
                            <div className="flex items-center gap-1 shrink-0">
                                <Badge variant="secondary" className="text-xs">
                                    {currentIndex + 1}/{pages.length}
                                </Badge>
                                <ChevronsUpDown className="h-4 w-4 opacity-50" />
                            </div>
                        </Button>
                    </PopoverTrigger>
                    <PopoverContent className="w-[400px] p-0" align="center">
                        <Command>
                            <CommandInput placeholder="Search pages by URL or title..." />
                            <CommandList>
                                <CommandEmpty>No pages found.</CommandEmpty>
                                <CommandGroup heading="Pages">
                                    {pages.map((p, idx) => (
                                        <CommandItem
                                            key={idx}
                                            value={`${p.url} ${p.title || ""}`}
                                            onSelect={() => {
                                                onNavigate(idx)
                                                setSearchOpen(false)
                                            }}
                                            className="cursor-pointer"
                                        >
                                            <div className="flex flex-col gap-0.5 flex-1 min-w-0">
                                                <span className="font-medium text-sm truncate">{getShortPath(p.url)}</span>
                                                <span className="text-xs text-muted-foreground truncate">{p.title || "No title"}</span>
                                            </div>
                                        </CommandItem>
                                    ))}
                                </CommandGroup>
                            </CommandList>
                        </Command>
                    </PopoverContent>
                </Popover>
            </div>

            <div className="flex gap-1">
                <Button
                    variant="outline"
                    size="icon"
                    onClick={goToPrev}
                    disabled={!canGoPrev}
                    className="h-8 w-8"
                    title="Previous page (←)"
                >
                    <ChevronLeft className="h-4 w-4" />
                </Button>
                <Button
                    variant="outline"
                    size="icon"
                    onClick={goToNext}
                    disabled={!canGoNext}
                    className="h-8 w-8"
                    title="Next page (→)"
                >
                    <ChevronRight className="h-4 w-4" />
                </Button>
            </div>
        </div>
    )
}
