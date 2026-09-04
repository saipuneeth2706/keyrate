"use client";

import Link from "next/link";

export function Navbar() {
  return (
    <header className="fixed top-0 right-0 left-0 z-50 flex items-center justify-between p-3 md:p-6">
      <Link href="/" className="shrink-0 hover:scale-105 motion-safe:transition">
        <div className="text-4xl text-slate-900">KeyRate</div>
      </Link>
    </header>
  );
}
