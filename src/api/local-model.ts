import { commands, type ModelInfo } from "@/src/bindings";
import { Result } from "@/src/lib/result";

export async function listLocalModels(): Promise<Result<ModelInfo[], string>> {
  const res = await commands.listLocalModels();
  return res.status === "ok" ? Result.Ok(res.data) : Result.Err(res.error ?? "");
}

export async function downloadLocalModel(modelId: string): Promise<Result<null, string>> {
  const res = await commands.downloadLocalModel(modelId);
  return res.status === "ok" ? Result.Ok(res.data) : Result.Err(res.error ?? "");
}

export async function cancelModelDownload(modelId: string): Promise<Result<null, string>> {
  const res = await commands.cancelModelDownload(modelId);
  return res.status === "ok" ? Result.Ok(res.data) : Result.Err(res.error ?? "");
}

export async function deleteLocalModel(modelId: string): Promise<Result<null, string>> {
  const res = await commands.deleteLocalModel(modelId);
  return res.status === "ok" ? Result.Ok(res.data) : Result.Err(res.error ?? "");
}

export async function getActiveLocalModel(): Promise<Result<string | null, string>> {
  const res = await commands.getActiveLocalModel();
  return res.status === "ok" ? Result.Ok(res.data) : Result.Err(res.error ?? "");
}

export async function setActiveLocalModel(modelId: string): Promise<Result<null, string>> {
  const res = await commands.setActiveLocalModel(modelId);
  return res.status === "ok" ? Result.Ok(res.data) : Result.Err(res.error ?? "");
}
