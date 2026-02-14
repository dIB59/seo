"use client"


import { useFormContext } from "react-hook-form"
import { FormField, FormItem, FormControl, FormMessage } from "@/src/components/ui/form"
import { Separator } from "@/src/components/ui/separator"
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
        <div className="space-y-6">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div className="space-y-4">
                    <h4 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground/70">Crawl Scope</h4>
                    <div className="grid grid-cols-1 gap-4">
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
                                        <p className="text-[10px] text-muted-foreground mt-1">
                                            Free tier limited to {maxPages} page.{" "}
                                            <span className="text-primary cursor-pointer hover:underline">Upgrade to Premium</span>
                                        </p>
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
                            control={control}
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
                            control={control}
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
                        control={control}
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
                        control={control}
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
    )
}
