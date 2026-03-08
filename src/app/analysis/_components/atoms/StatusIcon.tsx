import { cn } from "@/src/lib/utils"
import { CheckCircle2, XCircle } from "lucide-react"

export function StatusIcon({
    active,
    activeIcon: ActiveIcon,
    inactiveIcon: InactiveIcon,
}: {
    active: boolean
    activeIcon?: React.ComponentType<{ className?: string }>
    inactiveIcon?: React.ComponentType<{ className?: string }>
}) {
    const Icon = active ? ActiveIcon || CheckCircle2 : InactiveIcon || XCircle
    return <Icon className={cn("h-5 w-5", active ? "text-success" : "text-muted-foreground")} />
}