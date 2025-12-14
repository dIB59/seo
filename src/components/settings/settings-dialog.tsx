"use client"

import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"
import { toast } from "sonner"
import {
    DndContext,
    closestCenter,
    KeyboardSensor,
    PointerSensor,
    useSensor,
    useSensors,
    DragEndEvent
} from '@dnd-kit/core';
import {
    arrayMove,
    SortableContext,
    sortableKeyboardCoordinates,
    verticalListSortingStrategy,
    useSortable,
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';

import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
} from "@/src/components/ui/dialog"
import { Input } from "@/src/components/ui/input"
import { Label } from "@/src/components/ui/label"
import { Button } from "@/src/components/ui/button"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/src/components/ui/tabs"
import { Textarea } from "@/src/components/ui/textarea"
import { Trash2, Plus, GripVertical } from "lucide-react"
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
} from "@/src/components/ui/dropdown-menu"
import { Switch } from "@/src/components/ui/switch"

const DEFAULT_PERSONA = "You are an expert SEO consultant. Your tone is professional, encouraging, and data-driven."

interface PromptBlock {
    id: string
    type: "text" | "variable"
    content: string
}

const VARIABLE_OPTIONS = [
    { id: "url", label: "Website URL", template: "Website URL: {url}" },
    { id: "score", label: "SEO Score", template: "SEO Score: {score}" },
    { id: "pages", label: "Pages Analyzed", template: "Pages Analyzed: {pages_count}" },
    { id: "issues", label: "Total Issues Breakdown", template: "Total Issues: {total_issues}\n- Critical: {critical_issues}\n- Warnings: {warning_issues}" },
    { id: "top_issues", label: "Top 5 Issues List", template: "Top Issues Found:\n{top_issues}" },
    { id: "metrics", label: "Site Metrics", template: "Site Metrics:\n- Load Time: {avg_load_time}s\n- Total Words: {total_words}" },
    { id: "ssl", label: "SSL Status", template: "SSL Certificate: {ssl_certificate}" },
    { id: "sitemap", label: "Sitemap Status", template: "Sitemap Found: {sitemap_found}" },
    { id: "robots", label: "Robots.txt Status", template: "Robots.txt Found: {robots_txt_found}" },
]

function SortableBlock({ block, onRemove, onUpdate }: { block: PromptBlock, onRemove: (id: string) => void, onUpdate: (id: string, val: string) => void }) {
    const {
        attributes,
        listeners,
        setNodeRef,
        transform,
        transition,
    } = useSortable({ id: block.id });

    const style = {
        transform: CSS.Transform.toString(transform),
        transition,
    };

    return (
        <div ref={setNodeRef} style={style} className="flex gap-2 bg-muted/30 p-3 rounded-md border group mb-3 relative bg-card">
            {/* Controls */}
            <div className="flex flex-col gap-1 items-center justify-start pt-1">
                <div
                    {...attributes}
                    {...listeners}
                    className="bg-muted p-1 rounded cursor-grab active:cursor-grabbing text-muted-foreground hover:bg-muted-foreground/20 transition-colors"
                >
                    <GripVertical className="h-4 w-4" />
                </div>
            </div>

            {/* Content */}
            <div className="flex-1">
                <div className="flex justify-between items-center mb-1.5">
                    <span className={`text-[10px] uppercase font-bold tracking-wider px-1.5 rounded-sm ${block.type === 'variable' ? 'text-blue-600 bg-blue-50' : 'text-zinc-600 bg-zinc-100'}`}>
                        {block.type === 'variable' ? 'Dynamic Data' : 'Instruction Text'}
                    </span>
                    <Button variant="ghost" size="icon" className="h-6 w-6 text-muted-foreground hover:text-destructive" onClick={() => onRemove(block.id)}>
                        <Trash2 className="h-3.5 w-3.5" />
                    </Button>
                </div>
                <Textarea
                    value={block.content}
                    onChange={(e) => onUpdate(block.id, e.target.value)}
                    className={`text-xs font-mono bg-background resize-y ${block.type === 'text' ? 'min-h-[120px]' : 'min-h-[60px]'}`}
                    placeholder={block.type === 'variable' ? "Variable template (e.g. Score: {score})" : "Instructions..."}
                />
            </div>
        </div>
    );
}

export function SettingsDialog() {
    const [open, setOpen] = useState(false)
    const [apiKey, setApiKey] = useState("")
    const [persona, setPersona] = useState(DEFAULT_PERSONA)
    const [blocks, setBlocks] = useState<PromptBlock[]>([])
    const [aiEnabled, setAiEnabled] = useState(true)
    const [isLoading, setIsLoading] = useState(false)

    const sensors = useSensors(
        useSensor(PointerSensor),
        useSensor(KeyboardSensor, {
            coordinateGetter: sortableKeyboardCoordinates,
        })
    );

    useEffect(() => {
        const handleOpen = () => {
            setOpen(true)
            loadSettings()
        }

        window.addEventListener("open-settings-dialog", handleOpen)
        window.addEventListener("open-api-key-dialog", handleOpen)

        return () => {
            window.removeEventListener("open-settings-dialog", handleOpen)
            window.removeEventListener("open-api-key-dialog", handleOpen)
        }
    }, [])

    const loadSettings = async () => {
        try {
            const [key, savedPersona, savedBlocks, enabled] = await Promise.all([
                invoke<string | null>("get_gemini_api_key"),
                invoke<string | null>("get_gemini_persona"),
                invoke<string | null>("get_gemini_prompt_blocks"),
                invoke<boolean>("get_gemini_enabled")
            ])

            if (key) setApiKey(key)
            setPersona(savedPersona || DEFAULT_PERSONA)
            setAiEnabled(enabled)

            if (savedBlocks) {
                try {
                    const parsed = JSON.parse(savedBlocks)
                    if (Array.isArray(parsed)) setBlocks(parsed)
                } catch (e) {
                    console.error("Failed to parse prompt blocks", e)
                }
            } else {
                // Initial default blocks if none exist
                setBlocks([
                    { id: "intro", type: "text", content: "Please provide:\n1. A brief executive summary...\n\nData to include:" },
                    ...VARIABLE_OPTIONS.map(v => ({ id: v.id, type: "variable" as const, content: v.template }))
                ])
            }

        } catch (error) {
            console.error("Failed to load settings:", error)
            toast.error("Failed to load settings")
        }
    }

    const handleSaveApiKey = async () => {
        if (!apiKey || apiKey.trim().length === 0) {
            toast.error("Please enter a valid API Key")
            return
        }

        setIsLoading(true)
        try {
            await invoke("set_gemini_api_key", { apiKey: apiKey.trim() })
            toast.success("API Key saved successfully")
        } catch (error) {
            console.error("Error saving API key:", error)
            toast.error("Failed to save API Key")
        } finally {
            setIsLoading(false)
        }
    }

    const handleSaveGenericSettings = async () => {
        setIsLoading(true)
        try {
            await Promise.all([
                invoke("set_gemini_persona", { persona: persona.trim() }),
                invoke("set_gemini_enabled", { enabled: aiEnabled })
            ])
            toast.success("Configuration saved successfully")
        } catch (error) {
            console.error("Error saving generic settings:", error)
            toast.error("Failed to save configuration")
        } finally {
            setIsLoading(false)
        }
    }

    const handleSavePromptSettings = async () => {
        setIsLoading(true)
        try {
            await invoke("set_gemini_prompt_blocks", { blocks: JSON.stringify(blocks) })
            toast.success("Prompt layout saved successfully")
        } catch (error) {
            console.error("Error saving prompt settings:", error)
            toast.error("Failed to save prompt layout")
        } finally {
            setIsLoading(false)
        }
    }

    const handleResetDefaults = () => {
        setPersona(DEFAULT_PERSONA)
        setAiEnabled(true)
        setBlocks([
            { id: "intro", type: "text", content: "Please provide:\n1. A brief executive summary...\n\nData to include:" },
            ...VARIABLE_OPTIONS.map(v => ({ id: v.id, type: "variable" as const, content: v.template }))
        ])
    }

    // Block Management Functions
    const addTextBlock = () => {
        const newBlock: PromptBlock = {
            id: `text-${Date.now()}`,
            type: "text",
            content: ""
        }
        setBlocks([...blocks, newBlock])
    }

    const addVariableBlock = (template: string) => {
        const newBlock: PromptBlock = {
            id: `var-${Date.now()}`,
            type: "variable",
            content: template
        }
        setBlocks([...blocks, newBlock])
    }

    const removeBlock = (id: string) => {
        setBlocks(blocks.filter(b => b.id !== id))
    }

    const updateBlockContent = (id: string, content: string) => {
        setBlocks(blocks.map(b => b.id === id ? { ...b, content } : b))
    }

    const handleDragEnd = (event: DragEndEvent) => {
        const { active, over } = event;

        if (active.id !== over?.id) {
            setBlocks((items) => {
                const oldIndex = items.findIndex(item => item.id === active.id);
                const newIndex = items.findIndex(item => item.id === over?.id);

                return arrayMove(items, oldIndex, newIndex);
            });
        }
    }

    return (
        <Dialog open={open} onOpenChange={setOpen}>
            <DialogContent className="sm:max-w-[700px] max-h-[90vh] overflow-y-auto">
                <DialogHeader>
                    <DialogTitle>AI Analysis Configuration</DialogTitle>
                    <DialogDescription>
                        Configure Gemini AI API keys, expert persona, and prompt structure.
                    </DialogDescription>
                </DialogHeader>

                <Tabs defaultValue="general" className="w-full">
                    <TabsList className="grid w-full grid-cols-2">
                        <TabsTrigger value="general">Generic Settings</TabsTrigger>
                        <TabsTrigger value="prompt" disabled={!aiEnabled}>Prompt Builder</TabsTrigger>
                    </TabsList>

                    <TabsContent value="general" className="py-4 space-y-6">
                        {/* Enable Toggle */}
                        <div className="flex items-center justify-between space-x-2 border p-3 rounded-lg bg-secondary/20">
                            <div className="flex flex-col space-y-1">
                                <Label htmlFor="ai-mode" className="font-semibold">Enable AI Analysis</Label>
                                <span className="text-xs text-muted-foreground">
                                    When enabled, generates AI insights using Gemini Flash. Disabling skips this step.
                                </span>
                            </div>
                            <Switch id="ai-mode" checked={aiEnabled} onCheckedChange={setAiEnabled} />
                        </div>

                        <div className={`grid gap-2 transition-opacity ${!aiEnabled ? 'opacity-50 pointer-events-none' : ''}`}>
                            <Label htmlFor="apiKey">Gemini API Key</Label>
                            <div className="flex gap-2">
                                <Input
                                    id="apiKey"
                                    value={apiKey}
                                    onChange={(e) => setApiKey(e.target.value)}
                                    placeholder="AIza..."
                                    type="password"
                                />
                                <Button onClick={handleSaveApiKey} disabled={isLoading}>
                                    Save
                                </Button>
                            </div>
                            <p className="text-xs text-muted-foreground">
                                Get your free API key at <a href="https://makersuite.google.com/app/apikey" target="_blank" rel="noreferrer" className="text-primary hover:underline">Google AI Studio</a>.
                            </p>
                        </div>

                        {/* Persona Input (Moved here) */}
                        <div className={`grid gap-2 transition-opacity ${!aiEnabled ? 'opacity-50 pointer-events-none' : ''}`}>
                            <Label htmlFor="persona">AI Role & Tone</Label>
                            <Input
                                id="persona"
                                value={persona}
                                onChange={(e) => setPersona(e.target.value)}
                                placeholder="e.g. You are a strict SEO auditor..."
                            />
                        </div>

                        <div className="flex justify-end pt-4">
                            <Button onClick={handleSaveGenericSettings} disabled={isLoading}>
                                Save Configuration
                            </Button>
                        </div>
                    </TabsContent>

                    <TabsContent value="prompt" className={`py-4 space-y-4 transition-opacity ${!aiEnabled ? 'opacity-50 pointer-events-none' : ''}`}>
                        <div className="flex justify-between items-center mb-2">
                            <div className="text-xs text-muted-foreground">
                                Build your prompt by arranging blocks. The AI reads them top-to-bottom.
                            </div>
                            <Button variant="ghost" size="sm" onClick={handleResetDefaults} disabled={isLoading} className="h-8 text-xs">
                                Reset to Default
                            </Button>
                        </div>



                        {/* Prompt Blocks List - DRAGGABLE */}
                        <div className="space-y-3">
                            <DndContext
                                sensors={sensors}
                                collisionDetection={closestCenter}
                                onDragEnd={handleDragEnd}
                            >
                                <SortableContext
                                    items={blocks.map(b => b.id)}
                                    strategy={verticalListSortingStrategy}
                                >
                                    {blocks.map((block) => (
                                        <SortableBlock
                                            key={block.id}
                                            block={block}
                                            onRemove={removeBlock}
                                            onUpdate={updateBlockContent}
                                        />
                                    ))}
                                </SortableContext>
                            </DndContext>
                        </div>

                        {/* Add Buttons */}
                        <div className="flex gap-2 justify-center mt-4 border-t pt-4 border-dashed">
                            <Button variant="outline" size="sm" onClick={addTextBlock} className="h-8">
                                <Plus className="h-3 w-3 mr-1" /> Add Text Instruction
                            </Button>

                            <DropdownMenu>
                                <DropdownMenuTrigger asChild>
                                    <Button size="sm" className="h-8">
                                        <Plus className="h-3 w-3 mr-1" /> Add Data Variable
                                    </Button>
                                </DropdownMenuTrigger>
                                <DropdownMenuContent align="center">
                                    {VARIABLE_OPTIONS.map(opt => (
                                        <DropdownMenuItem key={opt.id} onClick={() => addVariableBlock(opt.template)}>
                                            {opt.label}
                                        </DropdownMenuItem>
                                    ))}
                                </DropdownMenuContent>
                            </DropdownMenu>
                        </div>

                        <div className="flex justify-end mt-4">
                            <Button onClick={handleSavePromptSettings} disabled={isLoading}>
                                Save Order & Content
                            </Button>
                        </div>
                    </TabsContent>
                </Tabs>
            </DialogContent>
        </Dialog>
    )
}
