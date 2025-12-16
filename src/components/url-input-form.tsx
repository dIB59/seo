"use client"

import type React from "react"

import { useState } from "react"
import { Globe, Play, Settings2 } from "lucide-react"
import { Button } from "@/src/components/ui/button"
import { Input } from "@/src/components/ui/input"
import { Label } from "@/src/components/ui/label"
import { Switch } from "@/src/components/ui/switch"
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/src/components/ui/collapsible"
import type { AnalysisSettingsRequest } from "@/src/lib/types"

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
  delay_between_requests: 500,
}

export function UrlInputForm({ onSubmit, isLoading }: UrlInputFormProps) {
  const [url, setUrl] = useState("")
  const [settings, setSettings] = useState<AnalysisSettingsRequest>(defaultSettings)
  const [showSettings, setShowSettings] = useState(false)

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (url.trim()) {
      onSubmit(url.trim(), settings)
      setUrl("")
    }
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div className="flex gap-3">
        <div className="relative flex-1">
          <Globe className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            type="url"
            placeholder="Enter website URL to analyze (e.g., https://example.com)"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            className="pl-10 bg-secondary border-border h-11"
            required
          />
        </div>
        <Button type="submit" disabled={isLoading || !url.trim()} className="h-11 px-6">
          <Play className="h-4 w-4 mr-2" />
          {isLoading ? "Starting..." : "Analyze"}
        </Button>
      </div>

      <Collapsible open={showSettings} onOpenChange={setShowSettings}>
        <CollapsibleTrigger asChild>
          <Button variant="ghost" size="sm" className="text-muted-foreground">
            <Settings2 className="h-4 w-4 mr-2" />
            Advanced Settings
          </Button>
        </CollapsibleTrigger>
        <CollapsibleContent className="mt-4">
          <div className="grid grid-cols-2 md:grid-cols-3 gap-4 p-4 bg-secondary/50 rounded-lg border border-border">
            <div className="space-y-2">
              <Label htmlFor="max-pages" className="text-sm">
                Max Pages
              </Label>
              <Input
                id="max-pages"
                type="number"
                value={settings.max_pages}
                onChange={(e) => setSettings({ ...settings, max_pages: Number.parseInt(e.target.value) || 100 })}
                className="bg-background"
                min={1}
                max={10000}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="delay" className="text-sm">
                Delay (ms)
              </Label>
              <Input
                id="delay"
                type="number"
                value={settings.delay_between_requests}
                onChange={(e) =>
                  setSettings({ ...settings, delay_between_requests: Number.parseInt(e.target.value) || 500 })
                }
                className="bg-background"
                min={0}
                max={5000}
              />
            </div>
            <div className="flex items-center justify-between space-x-2 col-span-2 md:col-span-1">
              <Label htmlFor="check-images" className="text-sm">
                Check Images
              </Label>
              <Switch
                id="check-images"
                checked={settings.check_images}
                onCheckedChange={(checked) => setSettings({ ...settings, check_images: checked })}
              />
            </div>
            <div className="flex items-center justify-between space-x-2">
              <Label htmlFor="external-links" className="text-sm">
                External Links
              </Label>
              <Switch
                id="external-links"
                checked={settings.include_external_links}
                onCheckedChange={(checked) => setSettings({ ...settings, include_external_links: checked })}
              />
            </div>
            <div className="flex items-center justify-between space-x-2">
              <Label htmlFor="mobile" className="text-sm">
                Mobile Analysis
              </Label>
              <Switch
                id="mobile"
                checked={settings.mobile_analysis}
                onCheckedChange={(checked) => setSettings({ ...settings, mobile_analysis: checked })}
              />
            </div>
            <div className="flex items-center justify-between space-x-2">
              <Label htmlFor="lighthouse" className="text-sm">
                Lighthouse
              </Label>
              <Switch
                id="lighthouse"
                checked={settings.lighthouse_analysis}
                onCheckedChange={(checked) => setSettings({ ...settings, lighthouse_analysis: checked })}
              />
            </div>
          </div>
        </CollapsibleContent>
      </Collapsible>
    </form>
  )
}
