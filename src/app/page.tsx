import type { Metadata } from "next";
import HomePageClient from "./HomePageClient";

export const metadata: Metadata = {
  title: "SEO Insikt crawler",
  description: "Analyze websites for SEO issues and get actionable recommendations",
};

export default function Home() {
  return <HomePageClient />;
}
