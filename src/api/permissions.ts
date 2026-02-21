import { commands, Policy } from "@/src/bindings";
import { wrapTauriCommand } from "./analysis";
import { Result } from "@/src/lib/result";

export async function getUserPolicy(): Promise<Result<Policy, string>> {
    return wrapTauriCommand(commands.getUserPolicy());
}
