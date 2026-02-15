import { Policy } from "@/src/bindings";
import { Badge } from "@/src/components/ui/badge";
import { Zap } from "lucide-react";

interface FeatureBadgesProps {
    policy: Policy;
}

export function FeatureBadges({ policy }: FeatureBadgesProps) {
    if (!policy.enabled_features.length) return null;

    return (
        <div className="space-y-3 pt-2">
            <h4 className="text-[10px] font-bold text-muted-foreground uppercase tracking-widest flex items-center gap-2">
                <Zap className="h-3 w-3 text-primary/80" />
                Enabled Capabilities
            </h4>
            <div className="flex flex-wrap gap-2">
                {policy.enabled_features.map((feature, index) => (
                    <div
                        key={feature}
                        className="animate-in fade-in zoom-in-95 slide-in-from-bottom-1 duration-300 fill-mode-both"
                        style={{ animationDelay: `${index * 50}ms` }}
                    >
                        <Badge
                            variant="secondary"
                            className="px-2.5 py-0.5 text-[10px] font-semibold bg-primary/5 text-primary border-primary/10 hover:bg-primary/10 transition-colors cursor-default select-none"
                        >
                            {feature.replace(/([A-Z])/g, ' $1').trim()}
                        </Badge>
                    </div>
                ))}
            </div>
        </div>
    );
}
