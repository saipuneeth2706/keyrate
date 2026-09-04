import type { Metadata } from "next";
import { Hero } from "@/components/Hero";

export const metadata: Metadata = {
  title: "KeyRate",
  description: "KeyRate - mechanical keyboard",
};

export default function Page() {
  return <Hero />;
}