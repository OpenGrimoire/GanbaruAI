<script lang="ts">
  import AppearanceSection from "./AppearanceSection.svelte";
  import ProfileSection from "./ProfileSection.svelte";
  import CalendarsSection from "./CalendarsSection.svelte";
  import NotesSection from "./NotesSection.svelte";
  import AboutSection from "./AboutSection.svelte";
  import FocusSection from "./FocusSection.svelte";
  import MobileSettingsStatus from "./mobile/MobileSettingsStatus.svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { SettingsSectionRendererProps } from "./settings-section-renderer-contract";

  let { activeSection }: SettingsSectionRendererProps = $props();
  const { t } = getLocalization();
</script>

{#if activeSection === "appearance"}
  <AppearanceSection />
{:else if activeSection === "profile"}
  <ProfileSection />
{:else if activeSection === "calendars"}
  <CalendarsSection fileTransfersAvailable={false} />
{:else if activeSection === "projects"}
  <MobileSettingsStatus
    heading={t("mobile.settings.projectsHeading")}
    description={t("mobile.settings.projectsDescription")}
  />
{:else if activeSection === "notes"}
  <NotesSection transfersAvailable={false} notificationsAvailable={false} />
{:else if activeSection === "chat"}
  <MobileSettingsStatus
    heading={t("mobile.settings.chatHeading")}
    description={t("mobile.settings.chatDescription")}
    detail={t("mobile.settings.chatDetail")}
  />
{:else if activeSection === "focus"}
  <FocusSection>
    {#snippet backgroundExecutionSettings()}
      {#await import("./AndroidBackgroundExecutionSettings.svelte") then module}
        {@const AndroidBackgroundExecutionSettings = module.default}
        <AndroidBackgroundExecutionSettings />
      {/await}
    {/snippet}
  </FocusSection>
{:else if activeSection === "music"}
  <MobileSettingsStatus
    heading={t("mobile.settings.musicHeading")}
    description={t("mobile.settings.musicDescription")}
  />
{:else if activeSection === "doomscrolling"}
  <MobileSettingsStatus
    heading={t("mobile.settings.doomscrollingHeading")}
    description={t("mobile.settings.doomscrollingDescription")}
  />
{:else if activeSection === "data"}
  <MobileSettingsStatus
    heading={t("mobile.privateDataHeading")}
    description={t("mobile.privateDataDescription")}
    detail={t("mobile.settings.dataDetail")}
  />
{:else if activeSection === "updates"}
  <MobileSettingsStatus
    heading={t("mobile.settings.updatesHeading")}
    description={t("mobile.settings.updatesDescription")}
  />
{:else if activeSection === "about"}
  <AboutSection />
{/if}
