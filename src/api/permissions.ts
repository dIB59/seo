import { commands } from "@/src/bindings";
import type { Feature, Policy } from "@/src/bindings";
import { wrapTauriCommand } from "./analysis";
import { Result } from "@/src/lib/result";

export type { Feature };

export async function getUserPolicy(): Promise<Result<Policy, string>> {
  return wrapTauriCommand(commands.getUserPolicy());
}
