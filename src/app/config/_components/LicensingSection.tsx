"use client";

import { useState, useEffect } from "react";
import { toast } from "sonner";
import { activate_license, get_license_tier, get_machine_id } from "@/src/api/licensing";
import { Button } from "@/src/components/ui/button";
import { Input } from "@/src/components/ui/input";
import { Label } from "@/src/components/ui/label";
import { CheckCircle2, Copy } from "lucide-react";

export function LicensingSection() {
    const [isLoading, setIsLoading] = useState(false);
    const [licenseKey, setLicenseKey] = useState("");
    const [tier, setTier] = useState<string | undefined>(undefined);
    const [machineId, setMachineId] = useState("");

    useEffect(() => {
        loadData();
    }, []);

    const loadData = async () => {
        const [tierRes, machineRes] = await Promise.all([
            get_license_tier(),
            get_machine_id()
        ]);
        if (tierRes.isOk()) setTier(tierRes.unwrap());
        if (machineRes.isOk()) setMachineId(machineRes.unwrap());
    };

    const handleActivate = async () => {
        if (!licenseKey.trim()) {
            toast.error("Please enter a key");
            return;
        }
        setIsLoading(true);
        try {
            const res = await activate_license(licenseKey.trim());
            if (res.isOk()) {
                setTier(res.unwrap());
                toast.success("License activated!");
                setLicenseKey("");
            } else {
                toast.error(res.unwrapErr());
            }
        } catch {
            toast.error("Activation failed");
        } finally {
            setIsLoading(false);
        }
    };

    const copyMachineId = () => {
        navigator.clipboard.writeText(machineId);
        toast.info("Machine ID copied");
    };

    return (
        <div className="space-y-6 animate-in fade-in slide-in-from-bottom-2 duration-300 max-w-xl">
            {/* Status Card */}
            <div className="bg-gradient-to-br from-card to-background p-6 rounded-xl border border-border/60 shadow-sm relative overflow-hidden">
                <div className="flex justify-between items-start z-10 relative">
                    <div>
                        <h3 className="text-sm font-medium text-muted-foreground uppercase tracking-wider">Current Plan</h3>
                        <div className="flex items-center gap-3 mt-1">
                            <span className="text-3xl font-bold">{tier || "..."}</span>
                            {tier === "Premium" && <CheckCircle2 className="h-5 w-5 text-green-500" />}
                        </div>
                    </div>
                </div>
                {/* Decorative background blur */}
                <div className="absolute -right-10 -top-10 w-32 h-32 bg-primary/2 rounded-full blur-3xl pointer-events-none" />
            </div>

            <div className="space-y-6 p-4 border border-border/50 rounded-lg bg-card/30 transition-all duration-300 hover:border-border/80">
                <div className="space-y-2">
                    <Label>Machine ID</Label>
                    <div className="flex gap-2">
                        <div className="flex-1 px-3 py-2 bg-muted/50 rounded-md border border-border/50 font-mono text-sm text-muted-foreground select-all">
                            {machineId || "Loading..."}
                        </div>
                        <Button variant="outline" size="icon" onClick={copyMachineId}>
                            <Copy className="h-4 w-4" />
                        </Button>
                    </div>
                    <p className="text-xs text-muted-foreground">
                        Unique hardware identifier for this device.
                    </p>
                </div>

                <div className="space-y-2 pt-4 border-t border-border/40">
                    <Label>Activate License</Label>
                    <div className="flex gap-2">
                        <Input
                            value={licenseKey}
                            onChange={(e) => setLicenseKey(e.target.value)}
                            placeholder="XXXX-XXXX-XXXX-XXXX"
                            className="font-mono bg-background"
                        />
                        <Button onClick={handleActivate} disabled={isLoading}>
                            Activate
                        </Button>
                    </div>
                </div>
            </div>
        </div>
    );
}
