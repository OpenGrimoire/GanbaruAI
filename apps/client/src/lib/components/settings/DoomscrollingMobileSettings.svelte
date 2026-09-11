<script lang="ts">
  import { onMount } from "svelte";
  import { cn } from "$lib/utils";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getDoomscrolling } from "$lib/stores/doomscrolling.svelte";
  import {
    mobileDoomscrollingAccessStatus,
    openMobileDoomscrollingAccessibilitySettings,
    openMobileDoomscrollingUsageAccessSettings,
    type MobileDoomscrollingAccessStatus,
  } from "$lib/scheduling/mobile-doomscrolling";
  import DoomscrollingConfigurationSection from "./DoomscrollingConfigurationSection.svelte";
  import DoomscrollingMobileAppSelector, {
    type DoomscrollingMobileAppSelection,
  } from "./DoomscrollingMobileAppSelector.svelte";
  import DoomscrollingRuleList from "./DoomscrollingRuleList.svelte";

  type ConfigurationToggle = "enabled" | "focus" | "shortBreaks" | "longBreaks" | "pause";
  type AccessTarget = "usage" | "accessibility";
  type PendingAction =
    | { target: "configuration"; toggle: ConfigurationToggle }
    | { target: "app"; type: "disable" | "delete"; packageName: string; name: string };

  const doomscrolling = getDoomscrolling();
  const { t } = getLocalization();
  let status = $state<MobileDoomscrollingAccessStatus | null>(null);
  let loadingStatus = $state(true);
  let statusError = $state(false);
  let pickerOpen = $state(false);
  let disclosure = $state<"accessibility" | null>(null);
  let pendingAction = $state<PendingAction | null>(null);

  const protectionActive = $derived(Boolean(status?.usageAccess && status.accessibility));
  const appItems = $derived(doomscrolling.blockedMobileApps.map((rule) => ({
    id: rule.packageName,
    label: rule.name,
    enabled: rule.enabled,
  })));

  async function refreshStatus(): Promise<void> {
    try {
      status = await mobileDoomscrollingAccessStatus();
      statusError = false;
    } catch (error) {
      console.warn("Android Doomscrolling access status failed", error);
      statusError = true;
    } finally {
      loadingStatus = false;
    }
  }

  async function openAccessSettings(target: AccessTarget | null): Promise<void> {
    try {
      if (target === "usage") {
        await openMobileDoomscrollingUsageAccessSettings();
      } else if (target === "accessibility") {
        await openMobileDoomscrollingAccessibilitySettings();
      }
    } catch (error) {
      console.warn("Android Doomscrolling settings failed", error);
      statusError = true;
    }
  }

  function reviewAccess(target: AccessTarget): void {
    if (target === "accessibility" && !status?.accessibility) {
      disclosure = target;
      return;
    }
    void openAccessSettings(target);
  }

  function agreeAndReview(): void {
    const target = disclosure;
    disclosure = null;
    void openAccessSettings(target);
  }

  function setConfiguration(toggle: ConfigurationToggle, checked: boolean): void {
    if (toggle === "enabled") doomscrolling.setMobileEnabled(checked);
    else if (toggle === "focus") doomscrolling.setMobileBlockDuringFocus(checked);
    else if (toggle === "shortBreaks") doomscrolling.setMobileBlockDuringShortBreaks(checked);
    else if (toggle === "longBreaks") doomscrolling.setMobileBlockDuringLongBreaks(checked);
    else doomscrolling.setMobilePauseDuringFocusPause(checked);
  }

  function requestConfigurationChange(toggle: ConfigurationToggle, checked: boolean): void {
    if (checked) setConfiguration(toggle, true);
    else pendingAction = { target: "configuration", toggle };
  }

  function requestAppChange(packageName: string, enabled: boolean): void {
    const app = doomscrolling.blockedMobileApps.find((rule) => rule.packageName === packageName);
    if (!app) return;
    if (enabled) doomscrolling.setBlockedMobileAppEnabled(packageName, true);
    else pendingAction = { target: "app", type: "disable", packageName, name: app.name };
  }

  function requestAppDelete(packageName: string): void {
    const app = doomscrolling.blockedMobileApps.find((rule) => rule.packageName === packageName);
    if (!app) return;
    pendingAction = { target: "app", type: "delete", packageName, name: app.name };
  }

  function confirmPendingAction(): void {
    if (!pendingAction) return;
    if (pendingAction.target === "configuration") {
      setConfiguration(pendingAction.toggle, false);
    } else if (pendingAction.type === "disable") {
      doomscrolling.setBlockedMobileAppEnabled(pendingAction.packageName, false);
    } else {
      doomscrolling.removeBlockedMobileApp(pendingAction.packageName);
    }
    pendingAction = null;
  }

  function pendingTitle(action: PendingAction): string {
    if (action.target === "app") {
      return action.type === "disable"
        ? t("settings.doomscrolling.mobile.allowAppTitle", action.name)
        : t("settings.doomscrolling.mobile.removeAppTitle", action.name);
    }
    if (action.toggle === "enabled") return t("settings.doomscrolling.mobile.turnOffTitle");
    if (action.toggle === "focus") return t("settings.doomscrolling.mobile.allowAppsFocusTitle");
    if (action.toggle === "shortBreaks") return t("settings.doomscrolling.mobile.allowAppsShortBreaksTitle");
    if (action.toggle === "longBreaks") return t("settings.doomscrolling.mobile.allowAppsLongBreaksTitle");
    return t("settings.doomscrolling.mobile.keepBlockingPausedTitle");
  }

  function pendingMessage(action: PendingAction): string {
    if (action.target === "app") {
      return action.type === "disable"
        ? t("settings.doomscrolling.mobile.appDisableMessage")
        : t("settings.doomscrolling.mobile.removeMessage");
    }
    if (action.toggle === "enabled") return t("settings.doomscrolling.mobile.appOffMessage");
    if (action.toggle === "focus") return t("settings.doomscrolling.mobile.focusOffMessage");
    if (action.toggle === "shortBreaks") return t("settings.doomscrolling.mobile.shortBreaksOffMessage");
    if (action.toggle === "longBreaks") return t("settings.doomscrolling.mobile.longBreaksOffMessage");
    return t("settings.doomscrolling.mobile.pauseActiveMessage");
  }

  onMount(() => {
    void refreshStatus();
    const refresh = (): void => {
      if (document.visibilityState === "visible") void refreshStatus();
    };
    window.addEventListener("focus", refresh);
    document.addEventListener("visibilitychange", refresh);
    return () => {
      window.removeEventListener("focus", refresh);
      document.removeEventListener("visibilitychange", refresh);
    };
  });
</script>

<div class="flex flex-col gap-6">
  <DoomscrollingConfigurationSection
    title={t("settings.doomscrolling.mobile.mobileConfiguration")}
    enabled={doomscrolling.mobileEnabled}
    blockDuringFocus={doomscrolling.mobileBlockDuringFocus}
    blockDuringShortBreaks={doomscrolling.mobileBlockDuringShortBreaks}
    blockDuringLongBreaks={doomscrolling.mobileBlockDuringLongBreaks}
    pauseDuringFocusPause={doomscrolling.mobilePauseDuringFocusPause}
    showMode={false}
    enabledLabel={t("settings.doomscrolling.mobile.enableMobileBlocking")}
    enabledDescription={t("settings.doomscrolling.mobile.enableMobileBlockingDescription")}
    focusDescription={t("settings.doomscrolling.mobile.focusDescription")}
    shortBreakDescription={t("settings.doomscrolling.mobile.shortBreakDescription")}
    longBreakDescription={t("settings.doomscrolling.mobile.longBreakDescription")}
    pauseDescription={t("settings.doomscrolling.mobile.pauseDescription")}
    onScheduleChange={requestConfigurationChange}
  />

  <div class="h-px shrink-0 scale-y-50 bg-border" aria-hidden="true"></div>

  <section class="flex flex-col gap-3">
    <h2 class="px-1 text-[0.866667rem] font-semibold text-foreground">{t("settings.doomscrolling.mobile.appBlocking")}</h2>
    <div class="flex flex-col gap-3">
      <div class="flex items-center justify-between gap-4 px-1 py-1">
        <div class="min-w-0 flex-1">
          <div class="text-[0.866667rem] text-foreground">{t("settings.doomscrolling.mobile.usageAccess")}</div>
          <div class="mt-0.5 text-[0.8rem] text-muted-foreground">{t("settings.doomscrolling.mobile.usageAccessDescription")}</div>
        </div>
        <button type="button" onclick={() => reviewAccess("usage")} class="h-7 shrink-0 rounded-md bg-primary px-2.5 text-[0.8rem] font-medium text-primary-foreground hover:bg-primary/90">{t("mobile.focusOnboarding.review")}</button>
      </div>
      <div class="flex items-center justify-between gap-4 px-1 py-1">
        <div class="min-w-0 flex-1">
          <div class="text-[0.866667rem] text-foreground">{t("settings.doomscrolling.mobile.appBlocking")}</div>
          <div class="mt-0.5 text-[0.8rem] text-muted-foreground">{t("settings.doomscrolling.mobile.appBlockingDescription")}</div>
        </div>
        <button type="button" onclick={() => reviewAccess("accessibility")} class="h-7 shrink-0 rounded-md bg-primary px-2.5 text-[0.8rem] font-medium text-primary-foreground hover:bg-primary/90">{t("mobile.focusOnboarding.review")}</button>
      </div>
    </div>
    <p class={cn("px-1 text-[0.8rem]", protectionActive ? "text-muted-foreground" : "text-destructive")}>
      {loadingStatus
        ? t("common.loading")
        : protectionActive
          ? t("settings.doomscrolling.mobile.protectionActive")
          : t("settings.doomscrolling.mobile.protectionInactive")}
    </p>
    {#if statusError}<p class="px-1 text-[0.8rem] text-destructive">{t("mobile.focusOnboarding.statusError")}</p>{/if}
  </section>

  <fieldset disabled={!doomscrolling.mobileEnabled} class={cn("m-0 flex min-w-0 flex-col gap-4 border-0 p-0", !doomscrolling.mobileEnabled && "opacity-50")}>
    <div class="h-px shrink-0 scale-y-50 bg-border" aria-hidden="true"></div>
    <section class="flex flex-col gap-4">
      <h2 class="px-1 text-[0.866667rem] font-semibold text-foreground">{t("settings.doomscrolling.mobile.blocklist")}</h2>
      <DoomscrollingRuleList
        id="doomscrolling-mobile-blocked-apps"
        heading={t("settings.doomscrolling.mobile.blockedApps")}
        description={t("settings.doomscrolling.mobile.blockedAppsDescription")}
        placeholder=""
        emptyText={t("settings.doomscrolling.mobile.noBlockedApps")}
        errorText=""
        items={appItems}
        onAdd={() => false}
        onOpenSelector={() => { pickerOpen = true; }}
        selectorLabel={t("settings.doomscrolling.mobile.addApp")}
        onEnabledChange={requestAppChange}
        onDelete={requestAppDelete}
      />
    </section>
  </fieldset>
</div>

{#if pickerOpen}
  <DoomscrollingMobileAppSelector
    title={t("settings.doomscrolling.mobile.chooseAppToBlock")}
    existingPackages={doomscrolling.blockedMobileApps.map((rule) => rule.packageName)}
    onAdd={(app: DoomscrollingMobileAppSelection) => doomscrolling.addBlockedMobileApp(app.name, app.packageName)}
    onRemove={(packageName) => doomscrolling.removeBlockedMobileApp(packageName)}
    onCancel={() => { pickerOpen = false; }}
  />
{/if}

{#if disclosure}
  <ConfirmDialog
    title={t("settings.doomscrolling.mobile.disclosureBlockingTitle")}
    message={t("settings.doomscrolling.mobile.disclosureBlockingMessage")}
    confirmLabel={t("settings.doomscrolling.mobile.agreeAndReview")}
    cancelLabel={t("settings.doomscrolling.mobile.notNow")}
    onConfirm={agreeAndReview}
    onCancel={() => { disclosure = null; }}
  />
{/if}

{#if pendingAction}
  <ConfirmDialog
    title={pendingTitle(pendingAction)}
    message={pendingMessage(pendingAction)}
    confirmLabel={pendingAction.target === "app" && pendingAction.type === "delete" ? t("settings.doomscrolling.shared.removeAction") : t("settings.doomscrolling.shared.allowAction")}
    cancelLabel={t("settings.doomscrolling.shared.cancelAction")}
    onConfirm={confirmPendingAction}
    onCancel={() => { pendingAction = null; }}
  />
{/if}
