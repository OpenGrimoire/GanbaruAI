# Music library and sources

## Canonical model

SQLite owns canonical media items, source collections, local roots and locations, playlist memberships, membership skip ranges, review state, snoozes, listening statistics, and playback state.

A media item represents stable content identity. Locations and source collections describe where it can currently be found. Playlist membership describes how that item behaves in one playlist. Source identity, physical location, and per-playlist playback settings remain separate canonical concerns.

## Local sources on desktop

Users select a folder and the app scans supported audio and video files without changing their location. The initial scan is bounded, deterministic, cancellable, and does not follow symbolic links. A newer scan supersedes an older scan safely.

The library stores a stable root plus relative location and file metadata suitable for detecting moved, missing, changed, or ambiguous media. Refresh preserves user-authored title, artist, album, artwork, review, snooze, and playlist choices when the underlying identity can still be established.

Missing files remain visible with a repair path. Relinking a source previews the effect and does not silently bind ambiguous matches.

## Local sources on Android

Android opens the system directory-tree picker, retains the granted read permission, and stores the selected tree identity plus relative document identities. It never fabricates desktop-style absolute paths.

The native scan is bounded by item count and traversal depth and runs outside the main thread. A lost permission is repaired by selecting the source folder again. Android currently advertises audio discovered through the selected tree, not general local video support.

## YouTube sources

Users can add supported YouTube video and playlist links. Playlist links resolve into canonical video items so Ganbaru AI owns queue order and navigation without requiring a YouTube Data API key.

Resolution has a timeout and explicit states for ready, unavailable, embedding blocked, and timed out. When metadata is available, the library stores the resolved title and appropriate source identity. Missing metadata retains a stable fallback label.

Owner-disabled embedding is never bypassed by scraping streams, proxying media, impersonating clients, or hidden download tools. The recovery action opens the video on YouTube.

## Artwork and metadata

Local artwork can come from a user override, a matching sidecar image, a common album-cover file, or supported embedded metadata. Explicit user overrides win. Artwork discovery remains bounded to the selected source hierarchy.

Missing or invalid artwork never prevents audio playback. Managed overrides follow vault asset ownership; source-owned sidecars remain at the music location.

## Source health

Sources report refresh state, availability, missing permission, missing files, ambiguous matches, unsupported media, and embedding failures. Source-wide refresh and relink actions live with source management rather than being duplicated across every playlist.

Repair never removes playlist membership or authored metadata merely because a source is temporarily offline.

## Search and transfer

Search is a rebuildable index over canonical titles, artists, albums, source collections, relative locations, and playlist metadata. It does not make indexed text authoritative.

Library transfer formats are versioned and bounded. They store source identity, playlists, membership configuration, and user-authored metadata without copying local media bytes. Import validates all untrusted values before canonical writes and reports skipped or unresolved items.

## Privacy

Local file names, paths, listening history, and playlist choices stay local. YouTube receives the requests required by its embedded player when the user loads a YouTube source. Ganbaru AI does not add remote analytics around those requests.
