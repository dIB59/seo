import React from "react";
import { Search, RefreshCw, Settings } from "lucide-react";
import { Button } from "@/src/components/ui/button";
import pkg from "@/package.json";

interface HeaderProps {
    isValidating: boolean;
    onRefresh: () => void;
}

export const HomeHeader = React.memo(function HomeHeader({
    isValidating,
    onRefresh,
}: HeaderProps) {
    return (
        <div className="flex flex-col gap-1 mb-8">
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                    <div className="flex items-center justify-center w-8 h-8 rounded-md bg-primary/10 text-primary border border-primary/20">
                        <Search className="h-4 w-4" />
                    </div>
                    <div>
                        <h1 className="text-lg font-semibold tracking-tight text-foreground/90">SEO Insikt</h1>
                    </div>
                    <div className="px-2 py-0.5 rounded-full bg-secondary text-[10px] font-mono text-muted-foreground border border-border/50">
                        v{pkg.version}
                    </div>
                </div>
                <div className="flex gap-2">
                    <Button
                        variant="ghost"
                        size="sm"
                        onClick={onRefresh}
                        disabled={isValidating}
                        className="h-8 text-muted-foreground hover:text-foreground"
                    >
                        <RefreshCw
                            className={`h-3.5 w-3.5 mr-2 ${isValidating ? "animate-spin" : ""}`}
                        />
                        Sync
                    </Button>
                    <Button
                        variant="outline"
                        size="sm"
                        onClick={() => window.location.href = '/config'}
                        className="h-8 bg-background/50 border-input/60 hover:bg-accent hover:text-accent-foreground"
                    >
                        <Settings className="h-3.5 w-3.5 mr-2" />
                        Config
                    </Button>
                </div>
            </div>
            <p className="text-sm text-muted-foreground/60 pl-[44px]">
                Advanced crawler and audit engine
            </p>
        </div>
    );
});
