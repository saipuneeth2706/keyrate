"use client";

import { useEffect, useRef, useState } from "react";
import { Canvas } from "@react-three/fiber";
import { Scene } from "./HeroScene";
import { Scales } from "@/components/ui/scales";
import { Tooltip } from "@/components/ui/tooltip-card";

const COMMAND = "npx keyrate@latest";

export function Hero() {
  const [mounted, setMounted] = useState(false);
  const [isDesktop, setIsDesktop] = useState(true);
  const [copied, setCopied] = useState(false);
  const copyTimeout = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    setMounted(true);
    const mq = window.matchMedia("(min-width: 768px)");
    const update = () => setIsDesktop(mq.matches);
    update();
    mq.addEventListener("change", update);
    return () => {
      mq.removeEventListener("change", update);
      if (copyTimeout.current) clearTimeout(copyTimeout.current);
    };
  }, []);

  const showKeyboard = mounted && isDesktop;

  const copyCommand = async () => {
    try {
      await navigator.clipboard.writeText(COMMAND);
    } catch {
      const textarea = document.createElement("textarea");
      textarea.value = COMMAND;
      textarea.style.position = "fixed";
      textarea.style.opacity = "0";
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand("copy");
      document.body.removeChild(textarea);
    }
    setCopied(true);
    if (copyTimeout.current) clearTimeout(copyTimeout.current);
    copyTimeout.current = setTimeout(() => setCopied(false), 2000);
  };

  return (
    <section className="relative h-dvh bg-[linear-gradient(to_bottom,#bfbcb0_0%,#dcd9d0_25%,#fdfdf9_55%)]">
      <div className="absolute inset-0 h-full w-full">
        <Scales orientation="vertical" size={12} color="rgba(148, 163, 184, 0.18)" />
      </div>

      {showKeyboard && (
        <div className="h-dvh w-full">
          <Canvas shadows="soft">
            <Scene />
          </Canvas>
        </div>
      )}

      <div className="hero-content flex h-dvh flex-col items-center justify-center gap-2 text-center md:absolute md:inset-x-0 md:bottom-0 md:h-auto md:justify-center md:gap-1 md:p-6 md:pb-8">
        <h1 className="text-3xl font-bold text-slate-900 md:text-4xl">
          Practise typing from your terminal
        </h1>
        <div className="max-w-md text-base text-slate-600">
          A typing{" "}
          <Tooltip
            containerClassName="text-slate-600 underline decoration-slate-400 underline-offset-2 cursor-help"
            content="TUI stands for Terminal User Interface. It lets you run the whole app right inside your terminal, using only your keyboard — no windows, no mouse needed."
          >
            <span className="font-medium">TUI</span>
          </Tooltip>{" "}
          built in{" "}
          <Tooltip
            containerClassName="text-slate-600 underline decoration-slate-400 underline-offset-2 cursor-help"
            content="Rust is a systems programming language known for speed and memory safety. It makes the typing experience fast and reliable without crashes."
          >
            <span className="font-medium">Rust</span>
          </Tooltip>{" "}
          with{" "}
          <Tooltip
            containerClassName="text-slate-600 underline decoration-slate-400 underline-offset-2 cursor-help"
            content="ratatui is a Rust library for building rich terminal user interfaces. It gives us flexible layouts, widgets, and smooth rendering in the terminal."
          >
            <span className="font-medium">ratatui</span>
          </Tooltip>{" "}
          and{" "}
          <Tooltip
            containerClassName="text-slate-600 underline decoration-slate-400 underline-offset-2 cursor-help"
            content="tachyonfx powers the silky animations you see, handling tweens and effects so motion in the terminal feels buttery smooth."
          >
            <span className="font-medium">tachyonfx</span>
          </Tooltip>{" "}
          for silky animations. Copy this command to get started.
        </div>
        <div className="mt-2 flex items-center border border-slate-300 bg-white/80 shadow-lg backdrop-blur-sm">
            <code className="px-4 py-3 text-sm font-medium text-slate-900">
              {COMMAND}
            </code>
          <button
            type="button"
            onClick={copyCommand}
            aria-label={copied ? "Copied" : "Copy command"}
            aria-live="polite"
            className="flex items-center border-l border-slate-300 px-3 py-3 text-slate-500 transition hover:bg-slate-100 hover:text-slate-900 active:bg-slate-200"
          >
            {copied ? (
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
                className="size-4 text-emerald-600"
              >
                <polyline points="20 6 9 17 4 12" />
              </svg>
            ) : (
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
                className="size-4"
              >
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
              </svg>
            )}
          </button>
        </div>
        <div className="mt-2 flex items-center justify-center gap-2 text-sm text-slate-500">
          <Tooltip
            containerClassName="inline-block"
            content="Portfolio website — click to open in a new tab"
          >
            <a
              href="https://www.saipuneeth.me"
              target="_blank"
              rel="noopener noreferrer"
              className="underline decoration-slate-400 underline-offset-2 transition hover:text-slate-900"
            >
              saipuneeth.me
            </a>
          </Tooltip>
          <span className="text-slate-400">|</span>
          <Tooltip
            containerClassName="inline-block"
            content="This is my Twitter/X handle. Click to open it in a new tab."
          >
            <a
              href="https://x.com/rsaipuneeth"
              target="_blank"
              rel="noopener noreferrer"
              className="underline decoration-slate-400 underline-offset-2 transition hover:text-slate-900"
            >
              @rsaipuneeth
            </a>
          </Tooltip>
        </div>
      </div>
    </section>
  );
}
