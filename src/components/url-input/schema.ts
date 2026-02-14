import * as z from "zod"
import type { AnalysisSettingsRequest } from "@/src/lib/types"

export const urlSchema = z.string().trim().min(1, "URL is required").refine((val) => {
    try {
        const toTest = /^https?:\/\//i.test(val) ? val : `https://${val}`
        const parsed = new URL(toTest)
        return parsed.hostname.includes(".") && (parsed.protocol === "http:" || parsed.protocol === "https:")
    } catch {
        return false
    }
}, "Invalid URL format")

export const createSchema = (maxPages: number) => z.object({
    url: urlSchema,
    settings: z.object({
        max_pages: z.number().min(1).max(maxPages, `Max pages limited to ${maxPages} on your current tier`),
        include_external_links: z.boolean(),
        check_images: z.boolean(),
        mobile_analysis: z.boolean(),
        lighthouse_analysis: z.boolean(),
        delay_between_requests: z.number().min(0).max(5000),
    })
})

// Keep baseSchema for type inference if needed, or derive it from a default
export const baseSchema = createSchema(10000)

export type FormValues = z.infer<typeof baseSchema>

export const defaultSettings: AnalysisSettingsRequest = {
    max_pages: 100,
    include_external_links: false,
    check_images: true,
    mobile_analysis: false,
    lighthouse_analysis: false,
    delay_between_requests: 5,
}

export const freeSettings: AnalysisSettingsRequest = {
    max_pages: 1,
    include_external_links: false,
    check_images: true,
    mobile_analysis: false,
    lighthouse_analysis: false,
    delay_between_requests: 5,
}

export const normalizeUrl = (input: string) => {
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
