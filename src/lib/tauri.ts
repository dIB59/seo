import { invoke } from "@tauri-apps/api/core"
import type { AnalysisProgress, AnalysisSettingsRequest, CompleteAnalysisResult } from "./types"
import { fromPromise } from "@/src/lib/result";



export const execute = <T>(command: string, args?: Record<string, unknown>) =>
	fromPromise(invoke<T>(command, args));

