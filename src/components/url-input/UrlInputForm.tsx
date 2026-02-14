"use client"

import React, { useCallback, useMemo } from "react"
import { useForm } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import type { AnalysisSettingsRequest } from "@/src/lib/types"
import { UrlInputGroup } from "./molecules/UrlInputGroup"
import { SettingsCollapsible } from "./molecules/SettingsCollapsible"
import { AnalysisSettingsFields } from "./organisms/AnalysisSettingsFields"
import { Form, FormField, FormItem, FormControl } from "@/src/components/ui/form"
import { usePermissions } from "@/src/hooks/use-permissions"
import { createSchema, defaultSettings, freeSettings, normalizeUrl, type FormValues } from "./schema"

interface UrlInputFormProps {
    onSubmit: (url: string, settings: AnalysisSettingsRequest) => void
    isLoading: boolean
}

export function UrlInputForm({ onSubmit, isLoading }: UrlInputFormProps) {
    const [showSettings, setShowSettings] = React.useState(false)
    const { maxPages, isFreeUser, isLoading: isPermissionsLoading } = usePermissions()

    const dynamicSchema = useMemo(() => createSchema(maxPages), [maxPages])

    const form = useForm<FormValues>({
        resolver: zodResolver(dynamicSchema),
        mode: "onChange",
        defaultValues: {
            url: "",
            settings: {
                ...defaultSettings,
                max_pages: Math.min(defaultSettings.max_pages, maxPages)
            },
        },
    })

    const { watch, setValue, handleSubmit, reset, formState } = form
    const currentSettings = watch("settings")

    React.useEffect(() => {
        if (!isPermissionsLoading) {
            const currentValues = form.getValues()
            reset({
                ...currentValues,
                settings: {
                    ...(isFreeUser ? freeSettings : defaultSettings),
                    // If user had manually changed something and we prefer to keep it, logic might differ.
                    // But here we reset to defaults on permission load/change to be safe and consistent.
                }
            })
        }
    }, [isPermissionsLoading, isFreeUser, reset, form])

    const isModified = JSON.stringify(currentSettings) !== JSON.stringify(isFreeUser ? freeSettings : defaultSettings)

    const handleReset = useCallback(() => {
        setValue("settings", isFreeUser ? freeSettings : defaultSettings, { shouldDirty: true, shouldValidate: true })
        // Re-enforce limit after reset if needed (though reset checks schema)
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

                <SettingsCollapsible
                    isOpen={showSettings}
                    onOpenChange={setShowSettings}
                    isModified={isModified}
                    onReset={handleReset}
                >
                    <AnalysisSettingsFields maxPages={maxPages} isFreeUser={isFreeUser} />
                </SettingsCollapsible>
            </form>
        </Form>
    )
}
