"use client"


import { useFormContext } from "react-hook-form"
import { FormField, FormItem, FormControl, FormMessage } from "@/src/components/ui/form"
import { SettingInput } from "../atoms/SettingInput"
import { SettingToggle } from "../atoms/SettingToggle"
import type { FormValues } from "../schema"

interface AnalysisSettingsFieldsProps {
    maxPages: number
    isFreeUser: boolean
}

export function AnalysisSettingsFields({ maxPages, isFreeUser }: AnalysisSettingsFieldsProps) {
    const { control } = useFormContext<FormValues>()

    return (
        <div className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-x-8 gap-y-6">
                {/* Column 1: Scope & Performance */}
                <div className="space-y-4">
                    <div className="flex items-center gap-2 pb-2 border-b border-border/40">
                        <h4 className="text-[10px] font-mono font-semibold uppercase tracking-wider text-muted-foreground">Scope & Limits</h4>
                    </div>

                    <div className="space-y-4">
                        <FormField
                            control={control}
                            name="settings.max_pages"
                            render={({ field }) => (
                                <FormItem>
                                    <FormControl>
                                        <SettingInput
                                            id="max-pages"
                                            label="Max Pages"
                                            tooltip="Total number of pages to crawl before stopping."
                                            value={field.value}
                                            onChange={field.onChange}
                                            min={1}
                                            max={maxPages}
                                            disabled={isFreeUser && maxPages === 1}
                                        />
                                    </FormControl>
                                    {isFreeUser && (
                                        <div className="flex items-center gap-1.5 mt-1.5 px-2 py-1 bg-amber-500/10 text-amber-600 rounded text-[10px] border border-amber-500/20 w-fit">
                                            <span>Free Tier Limit</span>
                                            <span className="font-mono">{maxPages}</span>
                                        </div>
                                    )}
                                    <FormMessage />
                                </FormItem>
                            )}
                        />
                        <FormField
                            control={control}
                            name="settings.delay_between_requests"
                            render={({ field }) => (
                                <SettingInput
                                    id="delay"
                                    label="Request Delay (ms)"
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

                {/* Column 2: Analysis Capabilities */}
                <div className="space-y-4">
                    <div className="flex items-center gap-2 pb-2 border-b border-border/40">
                        <h4 className="text-[10px] font-mono font-semibold uppercase tracking-wider text-muted-foreground">Capabilities</h4>
                    </div>

                    <div className="grid grid-cols-1 gap-3">
                        <FormField
                            control={control}
                            name="settings.lighthouse_analysis"
                            render={({ field }) => (
                                <SettingToggle
                                    id="deep-audit"
                                    label="Deep Audit"
                                    description="Full Lighthouse performance & SEO scan"
                                    tooltip="Runs a full Headless Chrome audit. Slower but provides detailed metrics."
                                    checked={field.value}
                                    onCheckedChange={field.onChange}
                                />
                            )}
                        />
                        <FormField
                            control={control}
                            name="settings.mobile_analysis"
                            render={({ field }) => (
                                <SettingToggle
                                    id="mobile"
                                    label="Mobile Emulation"
                                    description="Simulate mobile device viewport"
                                    checked={field.value}
                                    onCheckedChange={field.onChange}
                                />
                            )}
                        />
                        <div className="grid grid-cols-2 gap-3 pt-2">
                            <FormField
                                control={control}
                                name="settings.check_images"
                                render={({ field }) => (
                                    <SettingToggle
                                        id="check-images"
                                        label="Check Images"
                                        checked={field.value}
                                        onCheckedChange={field.onChange}
                                    />
                                )}
                            />
                            <FormField
                                control={control}
                                name="settings.include_external_links"
                                render={({ field }) => (
                                    <SettingToggle
                                        id="external-links"
                                        label="External Links"
                                        checked={field.value}
                                        onCheckedChange={field.onChange}
                                    />
                                )}
                            />
                        </div>
                    </div>
                </div>
            </div>
        </div>
    )
}
