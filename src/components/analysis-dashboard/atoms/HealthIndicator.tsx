import { CheckCircle2, AlertCircle } from "lucide-react";
import { StatusIcon } from "./StatusIcon";

export default function HealthIndicator({ label, active }: { label: string; active: boolean }) {
    return (
        <div className="flex flex-col items-center gap-1 p-2 rounded-lg bg-muted/50">
            <StatusIcon active={active} activeIcon={CheckCircle2} inactiveIcon={AlertCircle} />
            <span className="text-xs text-muted-foreground">{label}</span>
        </div>
    )
}