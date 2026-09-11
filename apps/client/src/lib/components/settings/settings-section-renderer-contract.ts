import type {
  ChatProviderSetupTarget,
  ChatSettingsSubsection,
  DoomscrollingLimitEditorTarget,
  DoomscrollingSettingsTab,
  NotesTransferOperation,
  SectionId,
} from "./types";

/** Props shared by the compile-time desktop and mobile settings renderers. */
export interface SettingsSectionRendererProps {
  readonly activeSection: SectionId;
  readonly initialDoomscrollingTab?: DoomscrollingSettingsTab;
  readonly activeChatSubsection: ChatSettingsSubsection;
  readonly initialChatTeammateId?: string;
  readonly initialChatChannelId?: string;
  readonly initialChatCreateTeammate?: boolean;
  readonly onOpenDoomscrollingLimitEditor: (target: DoomscrollingLimitEditorTarget) => void;
  readonly onOpenNotesTransferPanel: (operation: NotesTransferOperation) => void;
  readonly onOpenChatProviderSetup: (target: ChatProviderSetupTarget) => void;
  readonly onChatSubsectionChange: (subsection: ChatSettingsSubsection) => void;
  readonly onRequestNavigation: (navigate: () => void) => void;
  readonly onTeammateDraftStateChange: (open: boolean) => void;
}
