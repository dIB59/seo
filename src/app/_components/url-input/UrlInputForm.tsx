"use client";
"use no memo";

import React, { useCallback, useMemo } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import type { AnalysisSettingsRequest } from "@/src/api/analysis";
import { UrlInputGroup } from "./molecules/UrlInputGroup";
import { AnalysisSettingsCollapsible } from "./molecules/SettingsCollapsible";
import { AnalysisSettingsFields } from "./organisms/AnalysisSettingsFields";
import { Form, FormField, FormItem, FormControl } from "@/src/components/ui/form";
import { Skeleton } from "@/src/components/ui/skeleton";
import { usePermissions } from "@/src/hooks/use-permissions";
import { createSchema, normalizeUrl, type FormValues } from "./schema";
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

  const dynamicSchema = useMemo(() => createSchema(maxPages), [maxPages]);

  const form = useForm<FormValues>({
    resolver: zodResolver(dynamicSchema),
    mode: "onChange",
    defaultValues: {
      url: "",
      settings: {
        ...defaults,
        // Ensure max_pages doesn't exceed user's hard limit, even if default is higher (though backend should handle this)
        max_pages: Math.min(defaults.max_pages, maxPages),
      },
    },
  });

  const { watch, setValue, handleSubmit, reset, formState } = form;
  const currentSettings = watch("settings");

  // Calculate effective defaults for comparison (including the maxPage clamp)
  const effectiveDefaults = useMemo(
    () => ({
      ...defaults,
      max_pages: Math.min(defaults.max_pages, maxPages),
    }),
    [defaults, maxPages],
  );

  const isModified = JSON.stringify(currentSettings) !== JSON.stringify(effectiveDefaults);

  const handleReset = useCallback(() => {
    setValue("settings", effectiveDefaults, { shouldDirty: true, shouldValidate: true });
  }, [setValue, effectiveDefaults]);

  const onFormSubmit = (values: FormValues) => {
    const normalizedUrl = normalizeUrl(values.url);
    onSubmit(normalizedUrl, values.settings);
    reset({ url: "", settings: values.settings });
  };

  return (
    <Form {...form}>
      <form onSubmit={handleSubmit(onFormSubmit)} className="space-y-4">
        <FormField
          control={form.control}
          name="url"
          render={({ field }) => (
            <FormItem>
              <FormControl>
                <UrlInputGroup
                  url={field.value}
                  setUrl={field.onChange}
                  onClear={() => setValue("url", "", { shouldValidate: true })}
                  isLoading={isLoading}
                  isValid={formState.isValid}
                />
              </FormControl>
            </FormItem>
          )}
        />

        <AnalysisSettingsCollapsible
          isOpen={showSettings}
          onOpenChange={setShowSettings}
          isModified={isModified}
          onReset={handleReset}
        >
          <AnalysisSettingsFields maxPages={maxPages} isFreeUser={isFreeUser} />
        </AnalysisSettingsCollapsible>
      </form>
    </Form>
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
