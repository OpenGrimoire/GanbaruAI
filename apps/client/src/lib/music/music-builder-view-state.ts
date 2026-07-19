export interface MusicReviewTreeViewState {
  search: string;
  collapsedFolderIds: string[];
  selectedItemIds: string[];
  selectedFolderIds: string[];
  scrollTop: number;
}

export interface MusicReviewWorkspaceViewState {
  inlineCreateOpen: boolean;
  newPlaylistName: string;
  newPlaylistIcon: string;
  managingPlaylists: boolean;
  membershipBaselines: Record<string, string>;
  sessionSkippedIds: string[];
}

export interface MusicBuilderContextViewState {
  contextPanelOpen: boolean;
  selectedSourceId: string | null;
  issueFilter: import("$lib/music/music-issue-presentation").MusicIssueGroup | "all";
  issueExpandedGroups: import("$lib/music/music-issue-presentation").MusicIssueGroup[];
  soundscapeFilter: "all" | "generated" | "local";
}

export function createMusicReviewTreeViewState(): MusicReviewTreeViewState {
  return { search: "", collapsedFolderIds: [], selectedItemIds: [], selectedFolderIds: [], scrollTop: 0 };
}

export function createMusicReviewWorkspaceViewState(): MusicReviewWorkspaceViewState {
  return {
    inlineCreateOpen: false,
    newPlaylistName: "",
    newPlaylistIcon: "lucide:list-music",
    managingPlaylists: false,
    membershipBaselines: {},
    sessionSkippedIds: [],
  };
}

export function createMusicBuilderContextViewState(): MusicBuilderContextViewState {
  return {
    contextPanelOpen: false,
    selectedSourceId: null,
    issueFilter: "all",
    issueExpandedGroups: ["missing-local-file", "root-unavailable", "ambiguous-match", "youtube-unavailable", "embedding-blocked", "refresh-incomplete"],
    soundscapeFilter: "all",
  };
}
