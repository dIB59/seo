"use client"

import React, { useCallback, useMemo } from "react"
import { useForm } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import type { AnalysisSettingsRequest } from "@/src/lib/types"
import { UrlInputGroup } from "./molecules/UrlInputGroup"
import { AnalysisSettingsCollapsible } from "./molecules/SettingsCollapsible"
import { AnalysisSettingsFields } from "./organisms/AnalysisSettingsFields"
import { Form, FormField, FormItem, FormControl } from "@/src/components/ui/form"
import { Skeleton } from "@/src/components/ui/skeleton"
import { usePermissions } from "@/src/hooks/use-permissions"
import { createSchema, defaultSettings, freeSettings, normalizeUrl, type FormValues } from "./schema"

interface UrlInputFormProps {
    onSubmit: (url: string, settings: AnalysisSettingsRequest) => void
    isLoading: boolean
}

interface UrlInputFormContentProps extends UrlInputFormProps {
    maxPages: number
    isFreeUser: boolean
}

function UrlInputFormContent({ onSubmit, isLoading, maxPages, isFreeUser }: UrlInputFormContentProps) {
    const [showSettings, setShowSettings] = React.useState(false)

    const dynamicSchema = useMemo(() => createSchema(maxPages), [maxPages])

    const form = useForm<FormValues>({
        resolver: zodResolver(dynamicSchema),
        mode: "onChange",
        defaultValues: {
            url: "",
            settings: {
                ...(isFreeUser ? freeSettings : defaultSettings),
                max_pages: Math.min(defaultSettings.max_pages, maxPages)
            },
        },
    })

    const { watch, setValue, handleSubmit, reset, formState } = form
    const currentSettings = watch("settings")

    // Note: No useEffect needed for permission sync as this component mounts with correct defaults

    const isModified = JSON.stringify(currentSettings) !== JSON.stringify(isFreeUser ? freeSettings : defaultSettings)

    const handleReset = useCallback(() => {
        setValue("settings", isFreeUser ? freeSettings : defaultSettings, { shouldDirty: true, shouldValidate: true })
        // Re-enforce limit after reset if needed
        if (maxPages < defaultSettings.max_pages) {
            setValue("settings.max_pages", maxPages, { shouldValidate: true })
        }
    }, [setValue, maxPages, isFreeUser])

    const onFormSubmit = (values: FormValues) => {
        const normalizedUrl = normalizeUrl(values.url)
        onSubmit(normalizedUrl, values.settings)
        reset({ url: "", settings: values.settings })
    }

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
    )
}

export function UrlInputForm(props: UrlInputFormProps) {
    const { maxPages, isFreeUser, isLoading } = usePermissions()

    // Prevent rendering with wrong defaults by waiting for permissions
    if (isLoading) {
        return (
            <div className="space-y-4">
                <Skeleton className="h-[52px] w-full rounded-md" />
            </div>
        )
    }

    return <UrlInputFormContent {...props} maxPages={maxPages} isFreeUser={isFreeUser} />
}
