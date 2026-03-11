"use client";

import { SettingInput } from "../atoms/SettingInput";
import { SettingToggle } from "../atoms/SettingToggle";
import type { AnalysisSettingsRequest } from "@/src/api/analysis";

interface AnalysisSettingsFieldsProps {
  maxPages: number;
  isFreeUser: boolean;
  settings: AnalysisSettingsRequest;
  onSettingsChange: (next: Partial<AnalysisSettingsRequest>) => void;
}

export function AnalysisSettingsFields({
  maxPages,
  isFreeUser,
  settings,
  onSettingsChange,
}: AnalysisSettingsFieldsProps) {
  return (
    <div className="space-y-4">
      <div className="grid grid-cols-1 md:grid-cols-2 gap-x-8 gap-y-6">
        {/* Column 1: Scope & Performance */}
        <div className="space-y-4">
          <div className="flex items-center gap-2 pb-2 border-b border-border/40">
            <h4 className="text-[10px] font-mono font-semibold uppercase tracking-wider text-muted-foreground">
              Scope & Limits
            </h4>
          </div>

          <div className="space-y-4">
            <SettingInput
              id="max-pages"
              label="Max Pages"
              tooltip="Total number of pages to crawl before stopping."
              value={settings.max_pages}
              onChange={(value) =>
                onSettingsChange({ max_pages: Math.max(1, Math.min(value, maxPages)) })
              }
              min={1}
              max={maxPages}
              disabled={isFreeUser && maxPages === 1}
            />
            {isFreeUser && (
              <div className="flex items-center gap-1.5 mt-1.5 px-2 py-1 bg-amber-500/10 text-amber-600 rounded text-[10px] border border-amber-500/20 w-fit">
                <span>Free Tier Limit</span>
                <span className="font-mono">{maxPages}</span>
              </div>
            )}
            <SettingInput
              id="delay"
              label="Request Delay (ms)"
              tooltip="Pause between page requests to avoid hitting rate limits."
              value={settings.delay_between_requests}
              onChange={(value) =>
                onSettingsChange({ delay_between_requests: Math.max(0, Math.min(value, 5000)) })
              }
              min={0}
              max={5000}
            />
          </div>
        </div>

        {/* Column 2: Analysis Capabilities */}
        <div className="space-y-4">
          <div className="flex items-center gap-2 pb-2 border-b border-border/40">
            <h4 className="text-[10px] font-mono font-semibold uppercase tracking-wider text-muted-foreground">
              Capabilities
            </h4>
          </div>

          <div className="grid grid-cols-1 gap-3">
            <SettingToggle
              id="deep-audit"
              label="Deep Audit"
              description="Full Lighthouse performance & SEO scan"
              tooltip="Runs a full Headless Chrome audit. Slower but provides detailed metrics."
              checked={settings.lighthouse_analysis}
              onCheckedChange={(value) => onSettingsChange({ lighthouse_analysis: value })}
            />
            <SettingToggle
              id="mobile"
              label="Mobile Emulation"
              description="Simulate mobile device viewport"
              checked={settings.mobile_analysis}
              onCheckedChange={(value) => onSettingsChange({ mobile_analysis: value })}
            />
            <div className="grid grid-cols-2 gap-3 pt-2">
              <SettingToggle
                id="check-images"
                label="Check Images"
                checked={settings.check_images}
                onCheckedChange={(value) => onSettingsChange({ check_images: value })}
              />
              <SettingToggle
                id="subdomains"
                label="Subdomains"
                tooltip="Include links to subdomains as internal links"
                checked={settings.include_subdomains}
                onCheckedChange={(value) => onSettingsChange({ include_subdomains: value })}
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
