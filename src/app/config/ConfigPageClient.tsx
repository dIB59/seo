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
  set_gemini_enabled,
  set_gemini_prompt_blocks,
  set_gemini_api_key,
} from "@/src/api/ai";

import { useAiSettings } from "@/src/hooks/use-ai-settings";
import type { PromptBlock } from "@/src/lib/types";

// Components
import { ConfigSidebar, SIDEBAR_ITEMS } from "./_components/ConfigSidebar";
import { GeneralSettings } from "./_components/GeneralSettings";
import { PersonaSettings } from "./_components/PersonaSettings";
import { PromptBuilder } from "./_components/PromptBuilder";
import { LicensingSection } from "./_components/LicensingSection";
import { ThemeSettings } from "./_components/ThemeSettings";
import { ExtensionsSettings } from "./_components/ExtensionsSettings";

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
  const { settings, isLoading: isSwrLoading, mutate } = useAiSettings();

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

interface AiSettings {
  apiKey: string;
  persona: string;
  aiEnabled: boolean;
  blocks: PromptBlock[];
}

function ConfigContent({
  settings,
  mutate,
  activeSection,
}: {
  settings: AiSettings | undefined;
  mutate: (data?: AiSettings, options?: { revalidate: boolean }) => Promise<AiSettings | undefined>;
  activeSection: string;
}) {
  const [isLoading, setIsLoading] = useState(false);
  const [apiKey, setApiKey] = useState(settings?.apiKey || "");
  const [persona, setPersona] = useState(settings?.persona || "");
  const [aiEnabled, setAiEnabled] = useState(settings?.aiEnabled ?? true);
  const [blocks, setBlocks] = useState<PromptBlock[]>(settings?.blocks || []);

  const handleSaveGeneral = useCallback(async () => {
    setIsLoading(true);
    const [keyTask, personaTask, enabledTask] = await Promise.allSettled([
      set_gemini_api_key(apiKey),
      set_gemini_persona(persona),
      set_gemini_enabled(aiEnabled),
    ]);

    if (
      keyTask.status === "fulfilled" &&
      personaTask.status === "fulfilled" &&
      enabledTask.status === "fulfilled"
    ) {
      const keyRes = keyTask.value;
      const personaRes = personaTask.value;
      const enabledRes = enabledTask.value;

      if (keyRes.isOk() && personaRes.isOk() && enabledRes.isOk()) {
        if (settings) {
          mutate({ ...settings, apiKey, persona, aiEnabled }, { revalidate: false });
        }
        toast.success("Settings saved successfully");
      } else {
        toast.error("Failed to save some settings");
      }
    } else {
      toast.error("An error occurred while saving");
    }

    setIsLoading(false);
  }, [apiKey, persona, aiEnabled, mutate, settings]);

  const handleSavePrompt = useCallback(async () => {
    setIsLoading(true);
    const saveTask = await Promise.allSettled([set_gemini_prompt_blocks(JSON.stringify(blocks))]);

    if (saveTask[0].status === "fulfilled") {
      const res = saveTask[0].value;
      if (res.isOk()) {
        if (settings) {
          mutate({ ...settings, blocks }, { revalidate: false });
        }
        toast.success("Prompt layout saved");
      } else {
        toast.error("Failed to save layout");
      }
    } else {
      toast.error("Error saving layout");
    }

    setIsLoading(false);
  }, [blocks, mutate, settings]);

  // Keyboard Shortcut
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "s") {
        e.preventDefault();
        if (activeSection === "general" || activeSection === "persona") {
          handleSaveGeneral();
        } else if (activeSection === "prompt") {
          handleSavePrompt();
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [activeSection, handleSaveGeneral, handleSavePrompt]);

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
          <PersonaSettings persona={persona} setPersona={setPersona} />
        )}

        {activeSection === "prompt" && <PromptBuilder blocks={blocks} setBlocks={setBlocks} />}
        {activeSection === "extensions" && <ExtensionsSettings />}
        {activeSection === "licensing" && <LicensingSection />}
        {activeSection === "appearance" && <ThemeSettings />}
      </div>
    </>
  );
}
