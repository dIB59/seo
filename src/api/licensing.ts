import { commands, Policy } from "@/src/bindings";
import { Result } from "../lib/result";

export const activate_license = async (key: string): Promise<Result<Policy, string>> => {
    const res = await commands.activateWithKey(key);
    return res.status === "ok" ? Result.Ok(res.data) : Result.Err(res.error ?? "Failed to activate license");
}

export const get_license_tier = async (): Promise<Result<string, string>> => {
    const res = await commands.getLicenseTier();
    return res.status === "ok" ? Result.Ok(res.data) : Result.Err(res.error ?? "Failed to get license tier");
}

export const get_machine_id = async (): Promise<Result<string, string>> => {
    const res = await commands.getMachineId();
    return res.status === "ok" ? Result.Ok(res.data) : Result.Err(res.error ?? "Failed to get machine ID");
}
