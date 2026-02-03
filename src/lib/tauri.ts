import { invoke } from "@tauri-apps/api/core"
import { fromPromise } from "@/src/lib/result";



export const execute = <T>(command: string, args?: Record<string, unknown>) =>
	fromPromise(invoke<T>(command, args));

