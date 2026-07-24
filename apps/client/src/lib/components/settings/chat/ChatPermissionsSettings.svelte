<script lang="ts">
  import Settings from "@lucide/svelte/icons/settings";
  import ShieldAlert from "@lucide/svelte/icons/shield-alert";
  import ShieldCheck from "@lucide/svelte/icons/shield-check";
  import ShieldQuestionMark from "@lucide/svelte/icons/shield-question-mark";
  import type { Component } from "svelte";
  import { KNOWN_PROVIDER_FAMILIES, type SafetyMode } from "$lib/chat/contracts";
  import { providerSupportsPermissionMode } from "$lib/chat/permission-modes";
  import { getLocalization } from "$lib/i18n/translator.svelte";

  const { t } = getLocalization();
  const providerNames: Readonly<Record<(typeof KNOWN_PROVIDER_FAMILIES)[number], string>> = {
    codex: "Codex",
    claude: "Claude",
    cursor: "Cursor",
    grok: "Grok",
    opencode: "OpenCode",
  };
  const modes: ReadonlyArray<{
    value: SafetyMode;
    label: () => string;
    description: () => string;
    icon: Component;
  }> = [
    {
      value: "ask_for_approval",
      label: () => t("chat.hero.askForApproval"),
      description: () => t("chat.composer.askForApprovalDescription"),
      icon: ShieldQuestionMark,
    },
    {
      value: "approve_for_me",
      label: () => t("chat.hero.approveForMe"),
      description: () => t("chat.composer.approveForMeDescription"),
      icon: ShieldCheck,
    },
    {
      value: "full_access",
      label: () => t("chat.hero.fullAccess"),
      description: () => t("chat.composer.fullAccessShortDescription"),
      icon: ShieldAlert,
    },
    {
      value: "custom",
      label: () => t("chat.hero.customPermissions"),
      description: () => t("chat.composer.customPermissionsDescription"),
      icon: Settings,
    },
  ];

  function supportedProviders(mode: SafetyMode): string {
    return KNOWN_PROVIDER_FAMILIES
      .filter((familyId) => providerSupportsPermissionMode(familyId, mode))
      .map((familyId) => providerNames[familyId])
      .join(", ");
  }
</script>

<section class="flex flex-col gap-4">
  <div class="px-1">
    <h2 class="text-[0.866667rem] font-semibold text-foreground">{t("settings.chat.permissions.heading")}</h2>
    <p class="mt-1 text-[0.8rem] text-muted-foreground">{t("settings.chat.permissions.description")}</p>
  </div>

  <div class="grid gap-3 min-[560px]:grid-cols-2">
    {#each modes as mode (mode.value)}
      {@const Icon = mode.icon}
      <article class="rounded-lg border border-border bg-card/40 p-3">
        <div class="flex items-start gap-2.5">
          <span class="mt-0.5 grid size-7 shrink-0 place-items-center rounded-md bg-muted text-muted-foreground">
            <Icon size={16} strokeWidth={1.75} />
          </span>
          <div class="min-w-0">
            <h3 class="text-[0.8rem] font-semibold text-foreground">{mode.label()}</h3>
            <p class="mt-0.5 text-[0.733333rem] leading-5 text-muted-foreground">{mode.description()}</p>
          </div>
        </div>
        <p class="mt-2 border-t border-border/70 pt-2 text-[0.666667rem] text-muted-foreground">
          {t("settings.chat.permissions.availability", supportedProviders(mode.value))}
        </p>
      </article>
    {/each}
  </div>

  <div class="space-y-1 rounded-lg border border-border bg-muted/20 px-3 py-2.5 text-[0.733333rem] text-muted-foreground">
    <p>{t("settings.chat.permissions.selectedPerChat")}</p>
    <p>{t("settings.chat.permissions.unsupported")}</p>
    <p>{t("settings.chat.permissions.trust")}</p>
  </div>
</section>
