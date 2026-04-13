"use client";

import { useState } from "react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/src/components/ui/tabs";
import { ReportPatternsSettings } from "./ReportPatternsSettings";
import { ReportTemplateEditor } from "./ReportTemplateEditor";
import { PersonaSettings } from "./PersonaSettings";

interface ReportBuilderProps {
  persona: string;
  setPersona: (persona: string) => void;
}

export function ReportBuilder({ persona, setPersona }: ReportBuilderProps) {
  const [tab, setTab] = useState("patterns");

  return (
    <div className="space-y-6">
      <div>
        <p className="text-sm text-muted-foreground">
          Configure what your reports detect, how they're structured, and the AI
          voice that writes them.
        </p>
      </div>

      <Tabs value={tab} onValueChange={setTab} className="space-y-4">
        <TabsList>
          <TabsTrigger value="patterns">Patterns</TabsTrigger>
          <TabsTrigger value="template">Template</TabsTrigger>
          <TabsTrigger value="instructions">AI Instructions</TabsTrigger>
        </TabsList>

        <TabsContent value="patterns" className="space-y-4">
          <div className="text-sm text-muted-foreground mb-2">
            Patterns define <strong>what to detect</strong> across crawled pages.
            Each pattern is a rule that fires when a page field matches a
            condition above a prevalence threshold.
          </div>
          <ReportPatternsSettings />
        </TabsContent>

        <TabsContent value="template" className="space-y-4">
          <div className="text-sm text-muted-foreground mb-2">
            The template defines <strong>what the report says</strong>. Drag
            sections to reorder. AI sections are expanded by your local model
            at render time.
          </div>
          <ReportTemplateEditor />
        </TabsContent>

        <TabsContent value="instructions" className="space-y-4">
          <div className="text-sm text-muted-foreground mb-2">
            The persona is the <strong>system prompt</strong> prepended to every
            AI section in the template. It controls voice, tone, and guardrails.
          </div>
          <PersonaSettings persona={persona} setPersona={setPersona} />
        </TabsContent>
      </Tabs>
    </div>
  );
}
