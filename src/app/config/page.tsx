"use client";

import { useState, useEffect, useCallback } from "react";
import { Save } from "lucide-react";
import { Button } from "@/src/components/ui/button";
import { Separator } from "@/src/components/ui/separator";
import { Skeleton } from "@/src/components/ui/skeleton";
import { toast } from "sonner";
import { TooltipProvider } from "@/src/components/ui/tooltip";
import {
  set_gemini_persona,
  set_gemini_prompt_blocks,
} from "@/src/api/ai";

import { useAiSettings } from "@/src/hooks/use-ai-settings";
import type { PromptBlock } from "@/src/lib/types";

// Components
import { ConfigSidebar, SIDEBAR_ITEMS } from "./_components/ConfigSidebar";
import { AiSettings } from "./_components/AiSettings";
import { PersonaSettings } from "./_components/PersonaSettings";
import { PromptBuilder } from "./_components/PromptBuilder";
import { LicensingSection } from "./_components/LicensingSection";
import { ThemeSettings } from "./_components/ThemeSettings";
import { CustomChecksSettings } from "./_components/CustomChecksSettings";
import { ExtractorsSettings } from "./_components/ExtractorsSettings";
import { ReportPatternsSettings } from "./_components/ReportPatternsSettings";

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
  const { settings: rawSettings, isLoading: isSwrLoading, mutate: rawMutate } = useAiSettings();
  // Narrow to just what ConfigContent needs
  const settings = rawSettings ? { persona: rawSettings.persona, blocks: rawSettings.blocks } : undefined;
  const mutate = rawMutate as unknown as (data?: { persona: string; blocks: PromptBlock[] }, options?: { revalidate: boolean }) => Promise<{ persona: string; blocks: PromptBlock[] } | undefined>;

  const isInitialLoad = isSwrLoading && !settings;

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
              <ConfigContent
                key={settings ? "loaded" : "loading"}
                settings={settings}
                mutate={mutate}
                activeSection={activeSection}
              />
            )}
          </div>
        </main>
      </div>
    </TooltipProvider>
  );
}

interface PageSettings {
  persona: string;
  blocks: PromptBlock[];
}

function ConfigContent({
  settings,
  mutate,
  activeSection,
}: {
  settings: PageSettings | undefined;
  mutate: (data?: PageSettings, options?: { revalidate: boolean }) => Promise<PageSettings | undefined>;
  activeSection: string;
}) {
  const [isLoading, setIsLoading] = useState(false);
  const [persona, setPersona] = useState(settings?.persona || "");
  const [blocks, setBlocks] = useState<PromptBlock[]>(settings?.blocks || []);

  const handleSavePersona = useCallback(async () => {
    setIsLoading(true);
    try {
      const res = await set_gemini_persona(persona);
      if (res.isOk()) {
        if (settings) mutate({ ...settings, persona }, { revalidate: false });
        toast.success("Persona saved");
      } else {
        toast.error("Failed to save persona");
      }
    } catch {
      toast.error("An error occurred while saving");
    } finally {
      setIsLoading(false);
    }
  }, [persona, mutate, settings]);

  const handleSavePrompt = useCallback(async () => {
    setIsLoading(true);
    try {
      const res = await set_gemini_prompt_blocks(JSON.stringify(blocks));
      if (res.isOk()) {
        if (settings) mutate({ ...settings, blocks }, { revalidate: false });
        toast.success("Prompt layout saved");
      } else {
        toast.error("Failed to save layout");
      }
    } catch {
      toast.error("Error saving layout");
    } finally {
      setIsLoading(false);
    }
  }, [blocks, mutate, settings]);

  // ⌘S / Ctrl+S shortcut
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (!(e.metaKey || e.ctrlKey) || e.key !== "s") return;
      e.preventDefault();
      if (activeSection === "persona") handleSavePersona();
      else if (activeSection === "prompt") handleSavePrompt();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [activeSection, handleSavePersona, handleSavePrompt]);

  return (
    <>
      {/* Header */}
      <div className="flex items-center justify-between animate-in fade-in slide-in-from-top-4 duration-500">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">
            {SIDEBAR_ITEMS.find((i) => i.id === activeSection)?.label}
          </h2>
          <p className="text-muted-foreground">
            Manage your system configuration and AI preferences.
          </p>
        </div>
        {activeSection === "persona" && (
          <Button onClick={handleSavePersona} disabled={isLoading}>
            <Save className="h-4 w-4 mr-2" />
            Save
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
        {activeSection === "ai" && <AiSettings />}
        {activeSection === "persona" && (
          <PersonaSettings persona={persona} setPersona={setPersona} />
        )}
        {activeSection === "prompt" && <PromptBuilder blocks={blocks} setBlocks={setBlocks} />}
        {activeSection === "licensing" && <LicensingSection />}
        {activeSection === "appearance" && <ThemeSettings />}
        {activeSection === "custom-checks" && <CustomChecksSettings />}
        {activeSection === "custom-extractors" && <ExtractorsSettings />}
        {activeSection === "report-patterns" && <ReportPatternsSettings />}
      </div>
    </>
  );
}
