"use client";

import { useState, useEffect } from "react";
import { toast } from "sonner";
import { activate_license, get_machine_id } from "@/src/api/licensing";
import { getUserPolicy } from "@/src/api/permissions";
import { Policy } from "@/src/bindings";

// Sub-components
import { PlanStatusCard } from "./licensing/PlanStatusCard";
import { MachineIdBox } from "./licensing/MachineIdBox";
import { ActivationForm } from "./licensing/ActivationForm";

/**
 * LicensingSection - Main entry point for license management.
 * Follows "Refined Technical" aesthetic with modular architecture.
 */
export function LicensingSection() {
    const [isLoading, setIsLoading] = useState(false);
    const [policy, setPolicy] = useState<Policy | undefined>(undefined);
    const [machineId, setMachineId] = useState("");

    useEffect(() => {
        loadData();
    }, []);

    const loadData = async () => {
        const [policyRes, machineRes] = await Promise.all([
            getUserPolicy(),
            get_machine_id()
        ]);
        if (policyRes.isOk()) setPolicy(policyRes.unwrap());
        if (machineRes.isOk()) setMachineId(machineRes.unwrap());
    };

    const handleActivate = async (key: string) => {
        setIsLoading(true);
        try {
            const res = await activate_license(key);
            if (res.isOk()) {
                setPolicy(res.unwrap());
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
            setIsLoading(false);
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
                    <ActivationForm isLoading={isLoading} onActivate={handleActivate} />
                </div>
            </div>

            <p className="text-[9px] text-center text-muted-foreground/20 font-black tracking-[0.3em] uppercase pt-4">
                Rev 4.1 // {machineId?.slice(0, 12) || "NULL"}
            </p>
        </div>
    );
}
