import { Policy } from "@/src/bindings";
import { FeatureBadges } from "./FeatureBadges";

interface PlanStatusCardProps {
    policy: Policy | undefined;
}

export function PlanStatusCard({ policy }: PlanStatusCardProps) {
    const isPremium = policy?.tier === "Premium";

    const formatLimit = (limit: number) => {
        // Handle extremely large numbers (Specta/Rust UINT64_MAX or similar)
        if (limit > 1000000000000) return "Unlimited";
        return limit.toLocaleString();
    };

    return (
        <div className="relative group overflow-hidden rounded-xl border border-border/10">
            <div
                className={`
                    relative p-5 transition-all duration-500
                    animate-in fade-in slide-in-from-bottom-2
                    ${isPremium
                        ? "bg-primary/[0.02]"
                        : "bg-muted/[0.05]"}
                `}
            >
                <div className="relative z-10 space-y-5">
                    <div className="flex justify-between items-center">
                        <div className="space-y-0.5">
                            <h3 className="text-[9px] font-bold text-muted-foreground uppercase tracking-[0.2em] opacity-50">Current Tier</h3>
                            <div className="flex items-center gap-2">
                                <span className={`text-2xl font-black tracking-tight ${isPremium ? "text-primary italic" : "text-foreground"}`}>
                                    {policy?.tier || "..."}
                                </span>
                            </div>
                        </div>

                        {policy && (
                            <div className="flex gap-6">
                                <div className="flex flex-col items-end">
                                    <span className="text-[9px] font-bold text-muted-foreground/40 uppercase tracking-widest leading-none">Limit</span>
                                    <span className="text-lg font-bold tabular-nums leading-tight">{formatLimit(policy.max_pages)}</span>
                                </div>
                                <div className="flex flex-col items-end">
                                    <span className="text-[9px] font-bold text-muted-foreground/40 uppercase tracking-widest leading-none">Status</span>
                                    <div className="flex items-center gap-1.5 h-[22px]">
                                        <div className="h-1.5 w-1.5 rounded-full bg-green-500/80" />
                                        <span className="text-[10px] font-bold tracking-tight uppercase">OK</span>
                                    </div>
                                </div>
                            </div>
                        )}
                    </div>

                    {policy && <FeatureBadges policy={policy} />}
                </div>
            </div>
        </div>
    );
}
