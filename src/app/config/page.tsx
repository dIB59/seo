import type { Metadata } from "next";
import ConfigPageClient from "./ConfigPageClient";

export const metadata: Metadata = {
  title: "Configuration",
  description: "Manage AI settings, prompts, and extension configuration",
};

export default function ConfigPage() {
  return <ConfigPageClient />;
}
