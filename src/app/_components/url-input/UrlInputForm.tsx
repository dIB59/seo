"use client";

import React, { useMemo } from "react";
import type { AnalysisSettingsRequest } from "@/src/api/analysis";
import { UrlInputGroup } from "./molecules/UrlInputGroup";
import { AnalysisSettingsCollapsible } from "./molecules/SettingsCollapsible";
import { AnalysisSettingsFields } from "./organisms/AnalysisSettingsFields";
import { Skeleton } from "@/src/components/ui/skeleton";
import { usePermissions } from "@/src/hooks/use-permissions";
import { normalizeUrl } from "./schema";
import { useAnalysisDefaults } from "./use-analysis-defaults";

interface UrlInputFormProps {
  onSubmit: (url: string, settings: AnalysisSettingsRequest) => void;
  isLoading: boolean;
}

interface UrlInputFormContentProps extends UrlInputFormProps {
  maxPages: number;
  isFreeUser: boolean;
  defaults: AnalysisSettingsRequest;
}

function UrlInputFormContent({
  onSubmit,
  isLoading,
  maxPages,
  isFreeUser,
  defaults,
}: UrlInputFormContentProps) {
  const [showSettings, setShowSettings] = React.useState(false);

  const [url, setUrl] = React.useState("");
  const [settings, setSettings] = React.useState<AnalysisSettingsRequest>(() => ({
    ...defaults,
    max_pages: Math.min(defaults.max_pages, maxPages),
  }));

  // Calculate effective defaults for comparison (including the maxPage clamp)
  const effectiveDefaults = useMemo(
    () => ({
      ...defaults,
      max_pages: Math.min(defaults.max_pages, maxPages),
    }),
    [defaults, maxPages],
  );

  const isModified = JSON.stringify(settings) !== JSON.stringify(effectiveDefaults);
  const isValidUrl = url.trim().length > 0;

  const handleReset = () => setSettings(effectiveDefaults);

  const onFormSubmit = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const normalizedUrl = normalizeUrl(url);
    onSubmit(normalizedUrl, settings);
    setUrl("");
  };

  const updateSettings = (next: Partial<AnalysisSettingsRequest>) => {
    setSettings((previous) => ({ ...previous, ...next }));
  };

  return (
    <form onSubmit={onFormSubmit} className="space-y-4">
      <UrlInputGroup
        url={url}
        setUrl={setUrl}
        onClear={() => setUrl("")}
        isLoading={isLoading}
        isValid={isValidUrl}
      />

      <AnalysisSettingsCollapsible
        isOpen={showSettings}
        onOpenChange={setShowSettings}
        isModified={isModified}
        onReset={handleReset}
      >
        <AnalysisSettingsFields
          maxPages={maxPages}
          isFreeUser={isFreeUser}
          settings={settings}
          onSettingsChange={updateSettings}
        />
      </AnalysisSettingsCollapsible>
    </form>
  );
}

export function UrlInputForm(props: UrlInputFormProps) {
  const { maxPages, isFreeUser, isLoading } = usePermissions();
  const { defaults, isLoading: isLoadingDefaults } = useAnalysisDefaults(isFreeUser);

  // Prevent rendering with wrong defaults by waiting for permissions and defaults
  if (isLoading || isLoadingDefaults || !defaults) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-[52px] w-full rounded-md" />
      </div>
    );
  }

  return (
    <UrlInputFormContent
      {...props}
      maxPages={maxPages}
      isFreeUser={isFreeUser}
      defaults={defaults}
    />
  );
}
