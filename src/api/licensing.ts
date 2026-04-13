import { commands } from "@/src/bindings";
import type { Policy } from "@/src/bindings";
import { Result } from "../lib/result";

export type { Policy };

export async function activateLicense(key: string): Promise<Result<Policy, string>> {
  const res = await commands.activateWithKey(key);
  return res.status === "ok"
    ? Result.Ok(res.data)
    : Result.Err(res.error ?? "Failed to activate license");
}

export async function getLicenseTier(): Promise<Result<string, string>> {
  const res = await commands.getLicenseTier();
  return res.status === "ok"
    ? Result.Ok(res.data)
    : Result.Err(res.error ?? "Failed to get license tier");
}

export async function getMachineId(): Promise<Result<string, string>> {
  const res = await commands.getMachineId();
  return res.status === "ok"
    ? Result.Ok(res.data)
    : Result.Err(res.error ?? "Failed to get machine ID");
}
