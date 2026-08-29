<script lang="ts">
  import AppearanceSection from "./AppearanceSection.svelte";
  import ProfileSection from "./ProfileSection.svelte";
  import CalendarsSection from "./CalendarsSection.svelte";
  import ProjectsSection from "./ProjectsSection.svelte";
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
  <CalendarsSection fileTransfersAvailable={__GANBARU_AI_BUILD_PLATFORM__ === "android"} />
{:else if activeSection === "projects"}
  <ProjectsSection />
{:else if activeSection === "notes"}
  <NotesSection transfersAvailable={false} notificationsAvailable={false} />
{:else if activeSection === "chat"}
  {#await import("./mobile/MobileChatSection.svelte") then module}
    {@const MobileChatSection = module.default}
    <MobileChatSection />
  {/await}
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
  {#await import("./MusicSection.svelte") then module}
    {@const MusicSettings = module.default}
    <MusicSettings />
  {/await}
{:else if activeSection === "doomscrolling"}
  <MobileSettingsStatus
    heading={t("mobile.settings.doomscrollingHeading")}
    description={t("mobile.settings.doomscrollingDescription")}
  />
{:else if activeSection === "data"}
  {#await import("./mobile/MobileDataSection.svelte") then module}
    {@const MobileDataSection = module.default}
    <MobileDataSection />
  {/await}
{:else if activeSection === "updates"}
  {#await import("./mobile/MobileUpdatesSection.svelte") then module}
    {@const MobileUpdatesSection = module.default}
    <MobileUpdatesSection />
  {/await}
{:else if activeSection === "about"}
  <AboutSection />
{/if}
