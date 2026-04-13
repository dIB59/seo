import { commands } from "@/src/bindings";
import type {
  CustomCheck,
  CustomCheckParams,
  CustomExtractor,
  CustomExtractorParams,
  Tag,
  TagScope,
} from "@/src/bindings";

export type { CustomCheck, CustomCheckParams, CustomExtractor, CustomExtractorParams, Tag, TagScope };

// --- Tags ---

/** Fetch the full tag catalog, optionally filtered by scope. */
export async function listTags(scope?: TagScope): Promise<Tag[]> {
  const res = await commands.listTags(scope ?? null);
  if (res.status === "ok") return res.data;
  throw new Error(res.error ?? "Failed to list tags");
}

// --- Custom Checks ---

export async function listCustomChecks(): Promise<CustomCheck[]> {
  const res = await commands.listCustomChecks();
  if (res.status === "ok") return res.data;
  throw new Error(res.error ?? "Failed to list custom checks");
}

export async function createCustomCheck(params: CustomCheckParams): Promise<CustomCheck> {
  const res = await commands.createCustomCheck(params);
  if (res.status === "ok") return res.data;
  throw new Error(res.error ?? "Failed to create custom check");
}

export async function updateCustomCheck(id: string, params: CustomCheckParams): Promise<CustomCheck> {
  const res = await commands.updateCustomCheck(id, params);
  if (res.status === "ok") return res.data;
  throw new Error(res.error ?? "Failed to update custom check");
}

export async function deleteCustomCheck(id: string): Promise<void> {
  const res = await commands.deleteCustomCheck(id);
  if (res.status !== "ok") throw new Error(res.error ?? "Failed to delete custom check");
}

// --- Custom Extractors ---

export async function listCustomExtractors(): Promise<CustomExtractor[]> {
  const res = await commands.listCustomExtractors();
  if (res.status === "ok") return res.data;
  throw new Error(res.error ?? "Failed to list custom extractors");
}

export async function createCustomExtractor(params: CustomExtractorParams): Promise<CustomExtractor> {
  const res = await commands.createCustomExtractor(params);
  if (res.status === "ok") return res.data;
  throw new Error(res.error ?? "Failed to create custom extractor");
}

export async function updateCustomExtractor(id: string, params: CustomExtractorParams): Promise<CustomExtractor> {
  const res = await commands.updateCustomExtractor(id, params);
  if (res.status === "ok") return res.data;
  throw new Error(res.error ?? "Failed to update custom extractor");
}

export async function deleteCustomExtractor(id: string): Promise<void> {
  const res = await commands.deleteCustomExtractor(id);
  if (res.status !== "ok") throw new Error(res.error ?? "Failed to delete custom extractor");
}
