"use client"

import { useEffect, useCallback, useState } from "react"
import {
    ArrowLeft,
    ChevronLeft,
    ChevronRight,
    FileText,
    Clock,
    Link2,
    ImageIcon,
    Heading,
    ExternalLink,
    AlertCircle,
    CheckCircle2,
    Globe,
    Hash,
    Search,
    ChevronsUpDown,
    Gauge,
    Zap,
    Eye,
    Shield,
    Activity,
    LayoutPanelTop,
    MousePointer,
    XCircle,
} from "lucide-react"
import { Button } from "@/src/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/src/components/ui/card"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/src/components/ui/tabs"
import { Badge } from "@/src/components/ui/badge"
import { Separator } from "@/src/components/ui/separator"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/src/components/ui/table"
import {
    Command,
    CommandEmpty,
    CommandGroup,
    CommandInput,
    CommandItem,
    CommandList,
} from "@/src/components/ui/command"
import {
    Popover,
    PopoverContent,
    PopoverTrigger,
} from "@/src/components/ui/popover"
import {
    Tooltip,
    TooltipContent,
    TooltipProvider,
    TooltipTrigger,
} from "@/src/components/ui/tooltip"
import { cn } from "@/src/lib/utils"
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/src/components/ui/dialog"
import type { PageDetailData, HeadingElement, ImageElement, LinkElement } from "@/src/lib/types"
import { ScoreRing } from "../analysis-dashboard/atoms/ScoreRing"
import PageHeader from "./organisms/PageHeader"
import PageInfoCard from "./molecules/PageInfoCard"
import MetaTab from "./molecules/MetaTab"
import SeoAuditTab from "./molecules/SeoAuditTab"
import HeadingsTab from "./molecules/HeadingsTab"
import ImagesTab from "./molecules/ImagesTab"
import LinksTab from "./molecules/LinksTab"

// ============================================================================
// UTILITY COMPONENTS
// ============================================================================



// ============================================================================
// TAB COMPONENTS
// ============================================================================







// ============================================================================
// MAIN COMPONENT
// ============================================================================

interface PageDetailViewProps {
    page: PageDetailData
    pages: PageDetailData[]
    currentIndex: number
    onBack: () => void
    onNavigate: (index: number) => void
}

export function PageDetailView({ page, pages, currentIndex, onBack, onNavigate, }: PageDetailViewProps) {
    const [searchOpen, setSearchOpen] = useState(false)

    const canGoPrev = currentIndex > 0
    const canGoNext = currentIndex < pages.length - 1

    const goToPrev = useCallback(() => {
        if (canGoPrev) onNavigate(currentIndex - 1)
    }, [canGoPrev, currentIndex, onNavigate])

    const goToNext = useCallback(() => {
        if (canGoNext) onNavigate(currentIndex + 1)
    }, [canGoNext, currentIndex, onNavigate])

    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.target instanceof HTMLInputElement) return
            if (e.key === "ArrowLeft") goToPrev()
            if (e.key === "ArrowRight") goToNext()
            if (e.key === "Escape") onBack()
            if ((e.metaKey || e.ctrlKey) && e.key === "k") {
                e.preventDefault()
                setSearchOpen(true)
            }
        }
        window.addEventListener("keydown", handleKeyDown)
        return () => window.removeEventListener("keydown", handleKeyDown)
    }, [goToPrev, goToNext, onBack])

    return (
        <div className="space-y-4">
            <PageHeader page={page} pages={pages} currentIndex={currentIndex} onBack={onBack} onNavigate={onNavigate} />

            <PageInfoCard page={page} />

            <Tabs defaultValue="meta" className="space-y-4">
                <TabsList className="grid w-full grid-cols-5">
                    <TabsTrigger value="meta">Meta</TabsTrigger>
                    <TabsTrigger value="seo-audit">SEO Audit</TabsTrigger>
                    <TabsTrigger value="headings">Headings <Badge variant="secondary" className="ml-2 text-xs">{page.h1_count + page.h2_count + page.h3_count}</Badge></TabsTrigger>
                    <TabsTrigger value="images">Images <Badge variant="secondary" className="ml-2 text-xs">{page.image_count}</Badge></TabsTrigger>
                    <TabsTrigger value="links">Links <Badge variant="secondary" className="ml-2 text-xs">{page.internal_links + page.external_links}</Badge></TabsTrigger>
                </TabsList>

                <TabsContent value="meta"><MetaTab page={page} /></TabsContent>
                <TabsContent value="seo-audit"><SeoAuditTab page={page} /></TabsContent>
                <TabsContent value="headings"><HeadingsTab headings={page.headings || []} /></TabsContent>
                <TabsContent value="images"><ImagesTab images={page.images || []} /></TabsContent>
                <TabsContent value="links"><LinksTab links={page.detailed_links || []} /></TabsContent>
            </Tabs>

            <div className="text-center text-xs text-muted-foreground">
                <kbd className="px-1.5 py-0.5 bg-muted rounded text-[10px]">←</kbd>
                <kbd className="px-1.5 py-0.5 bg-muted rounded text-[10px] ml-1">→</kbd>
                <span className="ml-2">Navigate pages</span>
                <span className="mx-3">·</span>
                <kbd className="px-1.5 py-0.5 bg-muted rounded text-[10px]">Esc</kbd>
                <span className="ml-2">Back to results</span>
            </div>
        </div>
    )
}
