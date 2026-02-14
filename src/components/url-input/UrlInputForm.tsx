"use client"

import React, { useCallback, useMemo } from "react"
import { useForm } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import * as z from "zod"
import type { AnalysisSettingsRequest } from "@/src/lib/types"
import { UrlInputGroup } from "./molecules/UrlInputGroup"
import { SettingsCollapsible } from "./molecules/SettingsCollapsible"
import { SettingInput } from "./atoms/SettingInput"
import { SettingToggle } from "./atoms/SettingToggle"
import { Separator } from "@/src/components/ui/separator"
import { Form, FormField, FormItem, FormControl } from "@/src/components/ui/form"

const urlSchema = z.string().trim().min(1, "URL is required").refine((val) => {
    try {
        const toTest = /^https?:\/\//i.test(val) ? val : `https://${val}`
        const parsed = new URL(toTest)
        return parsed.hostname.includes(".") && (parsed.protocol === "http:" || parsed.protocol === "https:")
    } catch {
        return false
    }
}, "Invalid URL format")

const formSchema = z.object({
    url: urlSchema,
    settings: z.object({
        max_pages: z.number().min(1).max(10000),
        include_external_links: z.boolean(),
        check_images: z.boolean(),
        mobile_analysis: z.boolean(),
        lighthouse_analysis: z.boolean(),
        delay_between_requests: z.number().min(0).max(5000),
    })
})

type FormValues = z.infer<typeof formSchema>

interface UrlInputFormProps {
    onSubmit: (url: string, settings: AnalysisSettingsRequest) => void
    isLoading: boolean
}

const defaultSettings: AnalysisSettingsRequest = {
    max_pages: 100,
    include_external_links: false,
    check_images: true,
    mobile_analysis: false,
    lighthouse_analysis: false,
    delay_between_requests: 5,
}

export function UrlInputForm({ onSubmit, isLoading }: UrlInputFormProps) {
    const [showSettings, setShowSettings] = React.useState(false)
    const form = useForm<FormValues>({
        resolver: zodResolver(formSchema),
        mode: "onChange",
        defaultValues: {
            url: "",
            settings: defaultSettings,
        },
    })

    const { watch, setValue, handleSubmit, reset, formState } = form
    const currentSettings = watch("settings")

    const isModified = useMemo(() => {
        return JSON.stringify(currentSettings) !== JSON.stringify(defaultSettings)
    }, [currentSettings])

    const handleReset = useCallback(() => {
        setValue("settings", defaultSettings, { shouldDirty: true, shouldValidate: true })
    }, [setValue])

    const normalizeUrl = (input: string) => {
        const trimmed = input.trim()
        try {
            const hasProtocol = /^https?:\/\//i.test(trimmed)
            const toTest = hasProtocol ? trimmed : `https://${trimmed}`
            const parsed = new URL(toTest)
            return parsed.toString()
        } catch {
            return trimmed
        }
    }

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
                    <div className="space-y-6">
                        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                            <div className="space-y-4">
                                <h4 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground/70">Crawl Scope</h4>
                                <div className="grid grid-cols-1 gap-4">
                                    <FormField
                                        control={form.control}
                                        name="settings.max_pages"
                                        render={({ field }) => (
                                            <SettingInput
                                                id="max-pages"
                                                label="Max Pages"
                                                tooltip="Total number of pages to crawl before stopping."
                                                value={field.value}
                                                onChange={field.onChange}
                                                min={1}
                                                max={10000}
                                            />
                                        )}
                                    />
                                    <FormField
                                        control={form.control}
                                        name="settings.delay_between_requests"
                                        render={({ field }) => (
                                            <SettingInput
                                                id="delay"
                                                label="Delay (ms)"
                                                tooltip="Pause between page requests to avoid hitting rate limits."
                                                value={field.value}
                                                onChange={field.onChange}
                                                min={0}
                                                max={5000}
                                            />
                                        )}
                                    />
                                </div>
                            </div>

                            <div className="space-y-4">
                                <h4 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground/70">Analysis Features</h4>
                                <div className="space-y-3">
                                    <FormField
                                        control={form.control}
                                        name="settings.lighthouse_analysis"
                                        render={({ field }) => (
                                            <SettingToggle
                                                id="deep-audit"
                                                label="Deep Audit"
                                                description="Complete Lighthouse analysis"
                                                tooltip="Runs a full Headless Chrome audit. Slower but provides detailed performance and SEO scoring."
                                                checked={field.value}
                                                onCheckedChange={field.onChange}
                                            />
                                        )}
                                    />
                                    <FormField
                                        control={form.control}
                                        name="settings.mobile_analysis"
                                        render={({ field }) => (
                                            <SettingToggle
                                                id="mobile"
                                                label="Mobile View"
                                                description="Analyze as mobile device"
                                                checked={field.value}
                                                onCheckedChange={field.onChange}
                                            />
                                        )}
                                    />
                                </div>
                            </div>
                        </div>

                        <Separator className="bg-border/40" />

                        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                            <div className="space-y-3">
                                <FormField
                                    control={form.control}
                                    name="settings.check_images"
                                    render={({ field }) => (
                                        <SettingToggle
                                            id="check-images"
                                            label="Check Images"
                                            description="Detect missing alt tags & broken images"
                                            checked={field.value}
                                            onCheckedChange={field.onChange}
                                        />
                                    )}
                                />
                            </div>
                            <div className="space-y-3">
                                <FormField
                                    control={form.control}
                                    name="settings.include_external_links"
                                    render={({ field }) => (
                                        <SettingToggle
                                            id="external-links"
                                            label="Include External"
                                            description="Check status of outbound links"
                                            checked={field.value}
                                            onCheckedChange={field.onChange}
                                        />
                                    )}
                                />
                            </div>
                        </div>
                    </div>
                </SettingsCollapsible>
            </form>
        </Form>
    )
}
