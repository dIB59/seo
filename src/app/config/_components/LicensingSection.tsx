"use client";

import { useState } from "react";
import { toast } from "sonner";
import { activate_license } from "@/src/api/licensing";
import { usePermissions } from "@/src/hooks/use-permissions";

// Sub-components
import { PlanStatusCard } from "./licensing/PlanStatusCard";
import { MachineIdBox } from "./licensing/MachineIdBox";
import { ActivationForm } from "./licensing/ActivationForm";

/**
 * LicensingSection - Main entry point for license management.
 * Follows "Refined Technical" aesthetic with modular architecture.
 */
export function LicensingSection() {
    const [isActivating, setIsActivating] = useState(false);
    const { policy, machineId, mutate } = usePermissions();

    const handleActivate = async (key: string) => {
        setIsActivating(true);
        try {
            const res = await activate_license(key);
            if (res.isOk()) {
                const newPolicy = res.unwrap();
                mutate({ policy: newPolicy, machineId }, { revalidate: false });
                toast.success("Identity Verified", {
                    description: "Premium features have been unlocked for your device.",
                    duration: 4000,
                });
            } else {
                toast.error("Activation Error", {
                    description: res.unwrapErr(),
                });
            }
        } catch {
            toast.error("System Fault", {
                description: "Failed to reach licensing server. Please check connection.",
            });
        } finally {
            setIsActivating(false);
        }
    };

    return (
        <div className="space-y-6 max-w-xl pb-10">
            {/* Header branding */}
            <div className="space-y-0.5 px-0.5">
                <h2 className="text-lg font-bold tracking-tight uppercase tracking-tighter">Identity</h2>
                <p className="text-[11px] text-muted-foreground/50 font-medium">Hardware and licensing authorization</p>
            </div>

            {/* Main Sections */}
            <div className="space-y-6">
                <PlanStatusCard policy={policy} />

                <div className="space-y-6 pt-2">
                    <MachineIdBox machineId={machineId} />
                    <ActivationForm isLoading={isActivating} onActivate={handleActivate} />
                </div>
            </div>

            <p className="text-[9px] text-center text-muted-foreground/20 font-black tracking-[0.3em] uppercase pt-4">
                Rev 4.1 // {machineId?.slice(0, 12) || "NULL"}
            </p>
        </div>
    );
}
