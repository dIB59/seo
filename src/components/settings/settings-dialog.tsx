"use client"

import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"
import { toast } from "sonner"
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from "@/src/components/ui/dialog"
import { Input } from "@/src/components/ui/input"
import { Label } from "@/src/components/ui/label"
import { Button } from "@/src/components/ui/button"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/src/components/ui/tabs"
import { Textarea } from "@/src/components/ui/textarea"

const DEFAULT_PROMPT = `You are an expert SEO consultant. Analyze the following SEO audit results and provide actionable recommendations.

Website: {url}
SEO Score: {score}/100
Pages Analyzed: {pages_count}
Total Issues: {total_issues}
- Critical: {critical_issues}
- Warnings: {warning_issues}
- Suggestions: {suggestion_issues}

Top Issues Found:
{top_issues}

Site Metrics:
- Average Load Time: {avg_load_time}s
- Total Words: {total_words}
- SSL Certificate: {ssl_certificate}
- Sitemap Found: {sitemap_found}
- Robots.txt Found: {robots_txt_found}

Please provide:
1. A brief executive summary of the site's SEO health (2-3 sentences)
2. Top 5 priority actions the site owner should take, ranked by impact
3. Expected outcomes if these recommendations are implemented

Keep your response concise, actionable, and professional. Format for a PDF report.`

export function SettingsDialog() {
    const [open, setOpen] = useState(false)
    const [apiKey, setApiKey] = useState("")
    const [systemPrompt, setSystemPrompt] = useState("")
    const [isLoading, setIsLoading] = useState(false)

    useEffect(() => {
        const handleOpen = () => {
            setOpen(true)
            loadSettings()
        }

        window.addEventListener("open-settings-dialog", handleOpen)
        // Keep listener for backward compatibility or direct calls
        window.addEventListener("open-api-key-dialog", handleOpen)

        return () => {
            window.removeEventListener("open-settings-dialog", handleOpen)
            window.removeEventListener("open-api-key-dialog", handleOpen)
        }
    }, [])

    const loadSettings = async () => {
        try {
            const [key, prompt] = await Promise.all([
                invoke<string | null>("get_gemini_api_key"),
                invoke<string | null>("get_gemini_system_prompt")
            ])

            if (key) setApiKey(key)
            setSystemPrompt(prompt || DEFAULT_PROMPT)

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
            // Don't close dialog, let user continue editing if they want
        } catch (error) {
            console.error("Error saving API key:", error)
            toast.error("Failed to save API Key")
        } finally {
            setIsLoading(false)
        }
    }

    const handleSavePrompt = async () => {
        setIsLoading(true)
        try {
            await invoke("set_gemini_system_prompt", { prompt: systemPrompt.trim() })
            toast.success("System prompt saved successfully")
        } catch (error) {
            console.error("Error saving prompt:", error)
            toast.error("Failed to save prompt")
        } finally {
            setIsLoading(false)
        }
    }

    const handleResetPrompt = () => {
        setSystemPrompt(DEFAULT_PROMPT)
    }

    return (
        <Dialog open={open} onOpenChange={setOpen}>
            <DialogContent className="sm:max-w-[600px]">
                <DialogHeader>
                    <DialogTitle>Settings</DialogTitle>
                    <DialogDescription>
                        Configure your AI settings and API access.
                    </DialogDescription>
                </DialogHeader>

                <Tabs defaultValue="general" className="w-full">
                    <TabsList className="grid w-full grid-cols-2">
                        <TabsTrigger value="general">API Configuration</TabsTrigger>
                        <TabsTrigger value="prompt">AI Persona</TabsTrigger>
                    </TabsList>

                    <TabsContent value="general" className="py-4 space-y-4">
                        <div className="grid gap-2">
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
                    </TabsContent>

                    <TabsContent value="prompt" className="py-4 space-y-4">
                        <div className="grid gap-2">
                            <div className="flex justify-between items-center">
                                <Label htmlFor="prompt">System Prompt</Label>
                                <Button variant="ghost" size="sm" onClick={handleResetPrompt} disabled={isLoading} className="h-8 text-xs">
                                    Reset to Default
                                </Button>
                            </div>
                            <Textarea
                                id="prompt"
                                value={systemPrompt}
                                onChange={(e) => setSystemPrompt(e.target.value)}
                                placeholder="You are an expert SEO..."
                                className="min-h-[300px] font-mono text-xs"
                            />
                            <p className="text-xs text-muted-foreground">
                                Available variables: &#123;url&#125;, &#123;score&#125;, &#123;pages_count&#125;, &#123;total_issues&#125;, &#123;top_issues&#125;
                            </p>
                        </div>
                        <div className="flex justify-end">
                            <Button onClick={handleSavePrompt} disabled={isLoading}>
                                Save Changes
                            </Button>
                        </div>
                    </TabsContent>
                </Tabs>
            </DialogContent>
        </Dialog>
    )
}
