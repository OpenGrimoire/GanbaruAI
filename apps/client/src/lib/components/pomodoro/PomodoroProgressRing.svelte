<script lang="ts">
  import { cn } from "$lib/utils";

  let {
    active,
    remainingSeconds,
    totalSeconds,
    paused = false,
    pausedPulseAmount = null,
    size = 14.5,
    strokeWidth = 2.15,
    trackClass = "stroke-foreground/20 dark:stroke-white/20",
    progressClass = "text-foreground/68 stroke-foreground/68 dark:text-white/76 dark:stroke-white/76",
  }: {
    active: boolean;
    remainingSeconds: number;
    totalSeconds: number;
    paused?: boolean;
    pausedPulseAmount?: number | null;
    size?: number;
    strokeWidth?: number;
    trackClass?: string;
    progressClass?: string;
  } = $props();

  const circumference = 2 * Math.PI * 8;
  const remainingFraction = $derived.by(() => {
    if (!Number.isFinite(totalSeconds) || totalSeconds <= 0) return 0;
    if (!Number.isFinite(remainingSeconds)) return 0;
    return Math.min(1, Math.max(0, remainingSeconds / totalSeconds));
  });
  const pausedPulseStyle = $derived(
    paused && pausedPulseAmount !== null
      ? `--pomodoro-ring-paused-pulse-amount: ${Math.round(pausedPulseAmount * 100)}%;`
      : undefined,
  );
</script>

<svg
  data-pomodoro-progress-ring
  viewBox="0 0 20 20"
  width={size}
  height={size}
  aria-hidden="true"
>
  <circle
    cx="10"
    cy="10"
    r="8"
    fill="none"
    stroke-width={strokeWidth}
    class={trackClass}
  />
  {#if active}
    <circle
      data-pomodoro-progress-arc
      cx="10"
      cy="10"
      r="8"
      fill="none"
      stroke-width={strokeWidth}
      stroke-dasharray={`${remainingFraction * circumference} ${circumference}`}
      stroke-linecap="round"
      class={cn(`${progressClass} -rotate-90 origin-center`, paused ? "pomodoro-ring-paused-pulse" : "")}
      style={pausedPulseStyle}
    />
  {/if}
</svg>

<style>
  .pomodoro-ring-paused-pulse {
    stroke: color-mix(
      in srgb,
      color-mix(in srgb, currentColor 30%, transparent)
        var(--pomodoro-ring-paused-pulse-amount, 0%),
      currentColor
    );
  }

  @media (prefers-reduced-motion: reduce) {
    .pomodoro-ring-paused-pulse {
      stroke: color-mix(in srgb, currentColor 50%, transparent);
    }
  }
</style>
