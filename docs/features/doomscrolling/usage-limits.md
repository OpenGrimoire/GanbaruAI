# Doomscrolling usage limits

Usage limits are independent daily or weekly budgets. They apply whenever the relevant local adapter can observe a configured website or application, not only during a Pomodoro session.

## Limit model

A limit has a stable identity, optional display name, enabled state, at least one daily or weekly budget, and one or more linked entries. Entries can represent the same habit across platforms, such as a website host, Android package, and desktop application.

Single-entry limits can derive a display name from the entry. Multi-entry limits require a meaningful group name. New limits default to a one-hour daily budget and no weekly budget. Weekly budgets cannot exceed the full number of hours in a week.

Each linked source uses an event-palette color for the usage breakdown. Color communicates source allocation but is accompanied by labels.

Browser categories are not limit entries. Categories remain deterministic browser rule presets; limits target explicit hosts and applications.

## Time windows

Daily budgets reset at local midnight. Weekly budgets use the Monday-based local week. Timezone changes cause later samples to use the new local date; existing samples retain their recorded local date so a clock change does not silently rewrite history.

Deleting a limit does not delete usage samples. Recreating a matching limit can include already recorded activity in the current window, preventing delete-and-recreate from acting as a hidden reset.

## Browser counting

Browser usage counts the active HTTP or HTTPS tab while its browser window is focused. Passive focused use, including reading and fullscreen video, counts. A background tab, background browser window, locked browser, extension page, and unsupported browser URL do not count.

Samples contain normalized host, elapsed time, local date, and bounded timing metadata. They never contain the full URL, query string, page text, playback state, or input history.

## Android counting

Android counts explicitly selected packages while Android reports their activities visible and the screen interactive and unlocked. Passive use counts. Split-screen and picture-in-picture can count every selected package that remains visible.

Samples contain package identifier, display label, elapsed time, local date, and bounded timing. They do not contain screen content, text, taps, notifications, or unselected application history.

Legacy name-only entries remain visible but require reselection before package enforcement can be trusted.

## Desktop counting

Desktop counting prefers the foreground application where the operating system exposes it. Supported adapters can include Windows foreground windows, macOS frontmost applications, Linux X11 active-window data, and Wayland compositor protocols that expose focused applications.

On Wayland sessions without foreground visibility, a selected application may count while its safely matched process is open. The UI labels this as open-app time rather than focused use. If neither safe foreground nor process matching exists, the source is unavailable and does not accumulate time.

## Enforcement

When a browser budget is exhausted, matching top-level pages redirect to the extension block page. The page identifies whether the daily or weekly limit was reached and does not offer a bypass button.

On Android, an exhausted package-backed limit returns the user Home and posts a rate-limited notification linking to the limit settings.

Desktop application closing occurs only where a safe close adapter exists. The final boundary revalidates the exact enabled limit, linked entry, current total, app identity, protected-app policy, and observation freshness. A stale, removed, disabled, mismatched, or no-longer-exhausted limit performs no close.

## Presentation

Each budget displays used and available time with a stacked source breakdown. Limits that have both daily and weekly budgets show separate labeled groups. Unavailable adapters, stale permissions, and entries that require reselection are explicit.

Rounding and batching can produce a small delay before the newest interval appears. Presentation should not imply second-level precision when samples are intentionally batched.

## History and privacy

Usage history is local structured data. It supports totals and configuration guidance without becoming a complete activity diary. Retention and compaction preserve current budgets and useful trends while bounding storage.
