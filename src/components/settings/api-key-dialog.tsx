"use client"

import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"
import { toast } from "sonner"
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from "@/src/components/ui/dialog"
import { Input } from "@/src/components/ui/input"
import { Label } from "@/src/components/ui/label"
import { Button } from "@/src/components/ui/button"

export function ApiKeyDialog() {
    const [open, setOpen] = useState(false)
    const [apiKey, setApiKey] = useState("")
    const [isLoading, setIsLoading] = useState(false)

    useEffect(() => {
        // Listen for the custom event to open the dialog
        const handleOpen = () => {
            setOpen(true)
            // Optionally try to load existing key to pre-fill (though usually empty if we're asking)
            loadExistingKey()
        }

        window.addEventListener("open-api-key-dialog", handleOpen)
        return () => window.removeEventListener("open-api-key-dialog", handleOpen)
    }, [])

    const loadExistingKey = async () => {
        try {
            const key = await invoke<string | null>("get_gemini_api_key")
            if (key) {
                setApiKey(key)
            }
        } catch (error) {
            console.error("Failed to load existing key:", error)
        }
    }

    const handleSave = async () => {
        if (!apiKey || apiKey.trim().length === 0) {
            toast.error("Please enter a valid API Key")
            return
        }

        setIsLoading(true)
        try {
            await invoke("set_gemini_api_key", { apiKey: apiKey.trim() })
            toast.success("API Key saved successfully")
            setOpen(false)

            // Dispatch success event so caller can know we're done (optional, but good practice)
            window.dispatchEvent(new CustomEvent("api-key-saved"))

        } catch (error) {
            console.error("Error saving API key:", error)
            toast.error("Failed to save API Key")
        } finally {
            setIsLoading(false)
        }
    }

    return (
        <Dialog open={open} onOpenChange={setOpen}>
            <DialogContent className="sm:max-w-[425px]">
                <DialogHeader>
                    <DialogTitle>Gemini API Key Required</DialogTitle>
                    <DialogDescription>
                        To enable AI-powered insights, please provide your Google Gemini API Key.
                    </DialogDescription>
                </DialogHeader>
                <div className="grid gap-4 py-4">
                    <div className="grid grid-cols-4 items-center gap-4">
                        <Label htmlFor="apiKey" className="text-right">
                            API Key
                        </Label>
                        <Input
                            id="apiKey"
                            value={apiKey}
                            onChange={(e) => setApiKey(e.target.value)}
                            placeholder="AIza..."
                            className="col-span-3"
                            type="password"
                        />
                    </div>
                    <div className="text-xs text-muted-foreground text-center">
                        Don't have a key?{" "}
                        <a
                            href="https://makersuite.google.com/app/apikey"
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-primary hover:underline"
                        >
                            Get one here for free
                        </a>
                    </div>
                </div>
                <DialogFooter>
                    <Button variant="outline" onClick={() => setOpen(false)}>
                        Cancel
                    </Button>
                    <Button onClick={handleSave} disabled={isLoading}>
                        {isLoading ? "Saving..." : "Save API Key"}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    )
}
