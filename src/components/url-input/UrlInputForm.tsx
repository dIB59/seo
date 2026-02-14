"use client"

import React, { useState, useCallback, useMemo } from "react"
import type { AnalysisSettingsRequest } from "@/src/lib/types"
import { UrlInputGroup } from "./molecules/UrlInputGroup"
import { SettingsCollapsible } from "./molecules/SettingsCollapsible"
import { SettingInput } from "./atoms/SettingInput"
import { SettingToggle } from "./atoms/SettingToggle"
import { Separator } from "@/src/components/ui/separator"

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
    const [url, setUrl] = useState("")
    const [settings, setSettings] = useState<AnalysisSettingsRequest>(defaultSettings)
    const [showSettings, setShowSettings] = useState(false)

    const isModified = useMemo(() => {
        return JSON.stringify(settings) !== JSON.stringify(defaultSettings)
    }, [settings])

    const handleReset = useCallback(() => {
        setSettings(defaultSettings)
    }, [])

    const isValidUrl = useMemo(() => {
        const trimmed = url.trim()
        if (!trimmed) return false

        // Match either a full URL or a domain-like string (e.g. example.com)
        // This is a pragmatic check to enable the "Analyze" button
        const pattern = /^((https?:\/\/)|(www\.))?([a-z0-9]+([\\-\\.]{1}[a-z0-9]+)*\.[a-z]{2,5})(:[0-9]{1,5})?(\/.*)?$/i
        return pattern.test(trimmed)
    }, [url])

    const normalizeUrl = (input: string) => {
        let trimmed = input.trim()
        if (!trimmed) return ""

        // If it starts with www. or no protocol, prepend https://
        // If it starts with http://, keep it
        if (/^www\./i.test(trimmed) || !/^https?:\/\//i.test(trimmed)) {
            // Remove www prefix if we're adding https to avoid duplicates if user typed www.example.com
            trimmed = `https://${trimmed.replace(/^https?:\/\//i, "")}`
        }
        return trimmed
    }

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault()
        const normalizedUrl = normalizeUrl(url)
        if (normalizedUrl) {
            onSubmit(normalizedUrl, settings)
            setUrl("")
        }
    }

    return (
        <form onSubmit={handleSubmit} className="space-y-4">
            <UrlInputGroup
                url={url}
                setUrl={setUrl}
                onClear={() => setUrl("")}
                isLoading={isLoading}
                isValid={isValidUrl}
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
                                <SettingInput
                                    id="max-pages"
                                    label="Max Pages"
                                    tooltip="Total number of pages to crawl before stopping."
                                    value={settings.max_pages}
                                    onChange={(val) => setSettings(s => ({ ...s, max_pages: val }))}
                                    min={1}
                                    max={10000}
                                />
                                <SettingInput
                                    id="delay"
                                    label="Delay (ms)"
                                    tooltip="Pause between page requests to avoid hitting rate limits."
                                    value={settings.delay_between_requests}
                                    onChange={(val) => setSettings(s => ({ ...s, delay_between_requests: val }))}
                                    min={0}
                                    max={5000}
                                />
                            </div>
                        </div>

                        <div className="space-y-4">
                            <h4 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground/70">Analysis Features</h4>
                            <div className="space-y-3">
                                <SettingToggle
                                    id="deep-audit"
                                    label="Deep Audit"
                                    description="Complete Lighthouse analysis"
                                    tooltip="Runs a full Headless Chrome audit. Slower but provides detailed performance and SEO scoring."
                                    checked={settings.lighthouse_analysis}
                                    onCheckedChange={(checked) => setSettings(s => ({ ...s, lighthouse_analysis: checked }))}
                                />
                                <SettingToggle
                                    id="mobile"
                                    label="Mobile View"
                                    description="Analyze as mobile device"
                                    checked={settings.mobile_analysis}
                                    onCheckedChange={(checked) => setSettings(s => ({ ...s, mobile_analysis: checked }))}
                                />
                            </div>
                        </div>
                    </div>

                    <Separator className="bg-border/40" />

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                        <div className="space-y-3">
                            <SettingToggle
                                id="check-images"
                                label="Check Images"
                                description="Detect missing alt tags & broken images"
                                checked={settings.check_images}
                                onCheckedChange={(checked) => setSettings(s => ({ ...s, check_images: checked }))}
                            />
                        </div>
                        <div className="space-y-3">
                            <SettingToggle
                                id="external-links"
                                label="Include External"
                                description="Check status of outbound links"
                                checked={settings.include_external_links}
                                onCheckedChange={(checked) => setSettings(s => ({ ...s, include_external_links: checked }))}
                            />
                        </div>
                    </div>
                </div>
            </SettingsCollapsible>
        </form>
    )
}
