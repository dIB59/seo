import { commands, Policy } from "@/src/bindings";
import { Result } from "../lib/result";

export async function activate_license(key: string): Promise<Result<Policy, string>> {
    const res = await commands.activateWithKey(key);
    return res.status === "ok" ? Result.Ok(res.data) : Result.Err(res.error ?? "Failed to activate license");
}

export async function get_license_tier(): Promise<Result<string, string>> {
    const res = await commands.getLicenseTier();
    return res.status === "ok" ? Result.Ok(res.data) : Result.Err(res.error ?? "Failed to get license tier");
}

export async function get_machine_id(): Promise<Result<string, string>> {
    const res = await commands.getMachineId();
    return res.status === "ok" ? Result.Ok(res.data) : Result.Err(res.error ?? "Failed to get machine ID");
}
