import { Scales } from "@/components/ui/scales";

export default function ScalesVerticalDemo() {
  return (
    <div className="relative h-96 w-full overflow-hidden rounded-lg border border-neutral-200 bg-white dark:border-neutral-800 dark:bg-black">
      <Scales orientation="vertical" />
      <div className="relative z-10 flex h-full items-center justify-center">
        <h2 className="text-2xl font-bold text-neutral-900 dark:text-white">
          Vertical Scales
        </h2>
      </div>
    </div>
  );
}