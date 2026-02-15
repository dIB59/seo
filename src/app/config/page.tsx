"use client";

import { useState, useEffect, useCallback } from "react";
import { Save } from "lucide-react";
import { Button } from "@/src/components/ui/button";
import { Separator } from "@/src/components/ui/separator";
import { Skeleton } from "@/src/components/ui/skeleton";
import { toast } from "sonner";
import { TooltipProvider } from "@/src/components/ui/tooltip";
import {
    get_gemini_api_key,
    get_gemini_persona,
    get_gemini_enabled,
    set_gemini_persona,
    set_gemini_enabled,
    get_gemini_prompt_blocks,
    set_gemini_prompt_blocks,
    set_gemini_api_key
} from "@/src/api/ai";

// Components
import { ConfigSidebar, SIDEBAR_ITEMS } from "./_components/ConfigSidebar";
import { GeneralSettings } from "./_components/GeneralSettings";
import { PersonaSettings } from "./_components/PersonaSettings";
import { PromptBuilder, PromptBlock } from "./_components/PromptBuilder";
import { LicensingSection } from "./_components/LicensingSection";

function ContentSkeleton() {
    return (
        <div className="space-y-8 animate-pulse">
            <div className="flex items-center justify-between">
                <div className="space-y-2">
                    <Skeleton className="h-8 w-48" />
                    <Skeleton className="h-4 w-64" />
                </div>
                <Skeleton className="h-10 w-32" />
            </div>
            <Separator className="bg-border/40" />
            <div className="space-y-6">
                <Skeleton className="h-24 w-full rounded-lg" />
                <Skeleton className="h-40 w-full rounded-lg" />
            </div>
        </div>
    );
}

export default function ConfigPage() {
    const [activeSection, setActiveSection] = useState("general");
    const [isLoading, setIsLoading] = useState(false);
    const [isInitialLoad, setIsInitialLoad] = useState(true);

    // State
    const [apiKey, setApiKey] = useState("");
    const [persona, setPersona] = useState("");
    const [aiEnabled, setAiEnabled] = useState(true);
    const [blocks, setBlocks] = useState<PromptBlock[]>([]);

    useEffect(() => {
        loadSettings();
    }, []);

    const loadSettings = async () => {
        try {
            const [keyRes, personaRes, enabledRes, blocksRes] = await Promise.all([
                get_gemini_api_key(),
                get_gemini_persona(),
                get_gemini_enabled(),
                get_gemini_prompt_blocks()
            ]);

            if (keyRes.isOk()) setApiKey(keyRes.unwrap() || "");
            if (personaRes.isOk()) setPersona(personaRes.unwrap() || "");
            if (enabledRes.isOk()) setAiEnabled(enabledRes.unwrap());

            if (blocksRes.isOk()) {
                const saved = blocksRes.unwrap();
                if (saved) {
                    try {
                        const parsed = JSON.parse(saved);
                        if (Array.isArray(parsed)) setBlocks(parsed);
                    } catch {
                        console.error("Failed to parse blocks");
                    }
                }
            }

        } catch (error) {
            console.error("Failed to load settings:", error);
            toast.error("Failed to load settings");
        } finally {
            setIsInitialLoad(false);
        }
    };

    const handleSaveGeneral = useCallback(async () => {
        setIsLoading(true);
        try {
            const [keyRes, personaRes, enabledRes] = await Promise.all([
                set_gemini_api_key(apiKey),
                set_gemini_persona(persona),
                set_gemini_enabled(aiEnabled)
            ]);

            if (keyRes.isOk() && personaRes.isOk() && enabledRes.isOk()) {
                toast.success("Settings saved successfully");
            } else {
                toast.error("Failed to save some settings");
            }
        } catch {
            toast.error("An error occurred while saving");
        } finally {
            setIsLoading(false);
        }
    }, [apiKey, persona, aiEnabled]);

    const handleSavePrompt = useCallback(async () => {
        setIsLoading(true);
        try {
            const res = await set_gemini_prompt_blocks(JSON.stringify(blocks));
            if (res.isOk()) {
                toast.success("Prompt layout saved");
            } else {
                toast.error("Failed to save layout");
            }
        } catch {
            toast.error("Error saving layout");
        } finally {
            setIsLoading(false);
        }
    }, [blocks]);

    // Keyboard Shortcut: Ctrl+S / Cmd+S to save
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if ((e.metaKey || e.ctrlKey) && e.key === 's') {
                e.preventDefault();
                if (activeSection === 'general' || activeSection === 'persona') {
                    handleSaveGeneral();
                } else if (activeSection === 'prompt') {
                    handleSavePrompt();
                }
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [activeSection, handleSaveGeneral, handleSavePrompt]);

    return (
        <TooltipProvider>
            <div className="flex h-screen bg-background text-foreground">
                <ConfigSidebar activeSection={activeSection} setActiveSection={setActiveSection} />

                {/* Main Content */}
                <main className="flex-1 overflow-y-auto bg-background/50">
                    <div className="max-w-3xl mx-auto p-8 space-y-8">
                        {isInitialLoad ? (
                            <ContentSkeleton />
                        ) : (
                            <>
                                {/* Header */}
                                <div className="flex items-center justify-between animate-in fade-in slide-in-from-top-4 duration-500">
                                    <div>
                                        <h2 className="text-2xl font-bold tracking-tight">
                                            {SIDEBAR_ITEMS.find(i => i.id === activeSection)?.label}
                                        </h2>
                                        <p className="text-muted-foreground">
                                            Manage your system configuration and AI preferences.
                                        </p>
                                    </div>
                                    {(activeSection === "general" || activeSection === "persona") && (
                                        <Button onClick={handleSaveGeneral} disabled={isLoading}>
                                            <Save className="h-4 w-4 mr-2" />
                                            Save Changes
                                        </Button>
                                    )}
                                    {activeSection === "prompt" && (
                                        <Button onClick={handleSavePrompt} disabled={isLoading}>
                                            <Save className="h-4 w-4 mr-2" />
                                            Save Layout
                                        </Button>
                                    )}
                                </div>

                                <Separator className="bg-border/40" />

                                {/* Content Sections */}
                                <div className="space-y-6">
                                    {activeSection === "general" && (
                                        <GeneralSettings
                                            apiKey={apiKey}
                                            setApiKey={setApiKey}
                                            aiEnabled={aiEnabled}
                                            setAiEnabled={setAiEnabled}
                                        />
                                    )}

                                    {activeSection === "persona" && (
                                        <PersonaSettings
                                            persona={persona}
                                            setPersona={setPersona}
                                        />
                                    )}

                                    {activeSection === "prompt" && (
                                        <PromptBuilder blocks={blocks} setBlocks={setBlocks} />
                                    )}
                                    {activeSection === "licensing" && <LicensingSection />}
                                </div>
                            </>
                        )}
                    </div>
                </main>
            </div>
        </TooltipProvider>
    );
}
