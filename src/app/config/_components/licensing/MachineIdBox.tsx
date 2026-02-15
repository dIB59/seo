import { Copy } from "lucide-react";
import { toast } from "sonner";

interface MachineIdBoxProps {
    machineId: string;
}

export function MachineIdBox({ machineId }: MachineIdBoxProps) {
    const copyMachineId = () => {
        if (!machineId) return;
        navigator.clipboard.writeText(machineId);
        toast.success("ID Copied", {
            duration: 1500,
        });
    };

    return (
        <div className="space-y-3 group">
            <div className="px-1">
                <div className="space-y-0.5">
                    <h4 className="text-[9px] font-bold text-muted-foreground/50 uppercase tracking-widest">Device Identifier</h4>
                    <p className="text-[10px] text-muted-foreground/30 font-medium">Hardware bound to this installation</p>
                </div>
            </div>

            <div
                onClick={copyMachineId}
                className="w-full px-3 py-2.5 bg-muted/[0.03] rounded-lg border border-border/10 font-mono text-[11px] text-muted-foreground/80 cursor-pointer transition-all hover:border-primary/20 hover:text-foreground flex items-center justify-between group/id overflow-hidden"
            >
                <span className="truncate opacity-60 group-hover/id:opacity-100 transition-opacity">{machineId || "Generating..."}</span>
                <div className="flex items-center gap-3 ml-4 shrink-0 transition-all opacity-0 group-hover/id:opacity-100">
                    <span className="text-[8px] font-sans font-black uppercase tracking-tighter text-primary">Click to Copy</span>
                    <Copy className="h-3 w-3 text-primary/60" />
                </div>
            </div>
        </div>
    );
}
