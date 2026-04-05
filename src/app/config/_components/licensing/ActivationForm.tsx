import { Button } from "@/src/components/ui/button";
import { Textarea } from "@/src/components/ui/textarea";
import { Loader2 } from "lucide-react";
import { useState } from "react";

interface ActivationFormProps {
    isLoading: boolean;
    onActivate: (key: string) => Promise<void>;
}

export function ActivationForm({ isLoading, onActivate }: ActivationFormProps) {
    const [key, setKey] = useState("");

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!key.trim() || isLoading) return;
        await onActivate(key.trim());
        setKey("");
    };

    return (
        <form onSubmit={handleSubmit} className="space-y-4 pt-6 border-t border-border/10">
            <div className="space-y-2">
                <div className="flex items-center gap-2 mb-0.5 px-1">
                    <span className="text-[9px] font-bold text-muted-foreground/40 uppercase tracking-widest">Apply License Key</span>
                </div>
                <div className="space-y-2">
                    <Textarea
                        value={key}
                        onChange={(e) => setKey(e.target.value)}
                        placeholder="SEOINSIKT-..."
                        className="font-mono bg-transparent border-border/10 focus:border-primary/20 focus:ring-0 transition-all text-xs resize-none min-h-[60px]"
                        disabled={isLoading}
                        rows={2}
                    />
                    <Button
                        type="submit"
                        disabled={isLoading || !key.trim()}
                        className="w-full h-9 px-4 text-xs font-bold uppercase tracking-tight"
                    >
                        {isLoading ? (
                            <Loader2 className="h-3 w-3 animate-spin" />
                        ) : (
                            "Activate"
                        )}
                    </Button>
                </div>
            </div>
        </form>
    );
}
