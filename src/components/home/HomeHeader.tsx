import React from "react";
import { Search, RefreshCw, Settings } from "lucide-react";
import { Button } from "@/src/components/ui/button";

interface HeaderProps {
    isValidating: boolean;
    onRefresh: () => void;
    onOpenSettings: () => void;
}

export const HomeHeader = React.memo(function HomeHeader({
    isValidating,
    onRefresh,
    onOpenSettings,
}: HeaderProps) {
    return (
        <div className="flex items-center justify-between mb-8">
            <div className="flex items-center gap-3">
                <div className="p-2 bg-primary/20 rounded-lg">
                    <Search className="h-6 w-6 text-primary" />
                </div>
                <div>
                    <h1 className="text-2xl font-bold">SEO Insikt crawler</h1>
                    <p className="text-sm text-muted-foreground">
                        Analyze websites for SEO issues and recommendations
                    </p>
                </div>
            </div>
            <div className="flex gap-2">
                <Button
                    variant="ghost"
                    size="sm"
                    onClick={onRefresh}
                    disabled={isValidating}
                >
                    <RefreshCw
                        className={`h-4 w-4 mr-2 ${isValidating ? "animate-spin" : ""}`}
                    />
                    Refresh
                </Button>
                <Button variant="outline" size="sm" onClick={onOpenSettings}>
                    <Settings className="h-4 w-4 mr-2" />
                    AI Configuration
                </Button>
            </div>
        </div>
    );
});
