<script lang="ts">
  import type { Component } from "svelte";
  import AppearanceSection from "./AppearanceSection.svelte";
  import ProfileSection from "./ProfileSection.svelte";
  import CalendarsSection from "./CalendarsSection.svelte";
  import ProjectsSection from "./ProjectsSection.svelte";
  import NotesSection from "./NotesSection.svelte";
  import ChatSection from "./ChatSection.svelte";
  import FocusSection from "./FocusSection.svelte";
  import MusicSection from "./MusicSection.svelte";
  import DoomscrollingSection from "./DoomscrollingSection.svelte";
  import DataSection from "./DataSection.svelte";
  import UpdatesSection from "./UpdatesSection.svelte";
  import ShortcutsSection from "./ShortcutsSection.svelte";
  import AboutSection from "./AboutSection.svelte";
  import type { SectionId } from "./types";
  import type { SettingsSectionRendererProps } from "./settings-section-renderer-contract";

  let {
    activeSection,
    initialDoomscrollingTab,
    activeChatSubsection,
    initialChatTeammateId,
    initialChatChannelId,
    initialChatCreateTeammate,
    onOpenDoomscrollingLimitEditor,
    onOpenNotesTransferPanel,
    onOpenChatProviderSetup,
    onChatSubsectionChange,
    onRequestNavigation,
    onTeammateDraftStateChange,
  }: SettingsSectionRendererProps = $props();

  const BASIC_SECTION_COMPONENTS: Partial<Record<SectionId, Component>> = {
    appearance: AppearanceSection,
    profile: ProfileSection,
    calendars: CalendarsSection,
    projects: ProjectsSection,
    focus: FocusSection,
    music: MusicSection,
    data: DataSection,
    updates: UpdatesSection,
    shortcuts: ShortcutsSection,
    about: AboutSection,
  };

  const activeBasicSection = $derived(BASIC_SECTION_COMPONENTS[activeSection]);
</script>

{#if activeSection === "notes"}
  <NotesSection onOpenTransferPanel={onOpenNotesTransferPanel} />
{:else if activeSection === "doomscrolling"}
  <DoomscrollingSection
    initialTab={initialDoomscrollingTab}
    onOpenLimitEditor={onOpenDoomscrollingLimitEditor}
  />
{:else if activeSection === "chat"}
  <ChatSection
    initialSubsection={activeChatSubsection}
    {initialChatTeammateId}
    {initialChatChannelId}
    {initialChatCreateTeammate}
    onOpenProviderSetup={onOpenChatProviderSetup}
    onSubsectionChange={onChatSubsectionChange}
    {onRequestNavigation}
    {onTeammateDraftStateChange}
  />
{:else if activeBasicSection}
  {@const SectionComponent = activeBasicSection}
  <SectionComponent />
{/if}
