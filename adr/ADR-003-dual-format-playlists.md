## ADR-003: Dual-Format Playlist Persistence

* **Status:** Accepted
* **Date:** 2026-07-07
* **Author:** Architect

### Context

The audio player needs to persist collections of tracks that users can save and reload across sessions. The tracks may carry extended metadata: mix-in/mix-out points, per-song mix pattern overrides, and plugin parameter overrides. Two persistence strategies were considered:

**Option A — SQLite Library Database:** Store all tracks and playlists in a SQLite database. Provides querying, deduplication, and efficient random access. The user manages a single library.

**Option B — File-Based Playlists:** Store playlists as files. The user loads and saves playlist files manually — no central library. This means dual format: a primary JSON format for extended features and a secondary M3U8 format for cross-player compatibility.

Option B was chosen. The user explicitly wants no library database — playlists are the persistence mechanism. M3U8 compatibility is important for interoperability with other players and for users who manage their music directory manually. The JSON format carries everything the player needs.

### Decision

We will use a dual-format playlist system. The primary format is `.json` containing file paths, mix-in/mix-out points, mix pattern overrides, and any per-song metadata. The secondary format is `.m3u8` containing only file paths for cross-player compatibility. The JSON file is self-contained and does not require a companion M3U8. Users can export to M3U8 from the UI. Deleted tracks are removed from the playlist on next save.

### Visual Architecture

```mermaid
graph TD
    subgraph Import Sources
        A[Drag-Drop Files] --> D[In-Memory Playlist]
        B[Drag-Drop Folder] --> D
        C[Load Directory Dialog] --> D
    end

    subgraph Persistence
        D --> E[Save as JSON]
        D --> F[Save as M3U8]
        G[Load JSON] --> D
        H[Load M3U8] --> D
    end

    subgraph JSON Format
        I[{<br/>  "version": 1,<br/>  "tracks": [<br/>    {<br/>      "path": "...",<br/>      "mix_out": 180.0,<br/>      "mix_in": 30.0,<br/>      "mix_pattern": "cross-fade"<br/>    }<br/>  ]<br/>}]
    end

    subgraph M3U8 Format
        J[#EXTM3U<br/>#EXTINF:301,Artist - Song<br/>/path/to/file.mp3<br/>#EXTINF:245,Artist2 - Song2<br/>/path/to/file2.flac]
    end

    subgraph Actions
        K[Delete: remove from playlist]
        L[DeletePlus: remove from<br/>playlist + delete from disk]
        K --> D
        L --> D
    end
```

### Consequences

**Positive (Benefits):**
- No database schema, migrations, or sync complexity.
- Playlist files are portable — can be moved between machines, shared, edited by hand.
- M3U8 is an open format readable by virtually all media players.
- Users manage their own file organization — no vendor lock-in.

**Negative (Risks/Trade-offs):**
- No cross-playlist search or indexing — search is scoped to the current playlist.
- Duplicate tracks across playlists mean duplicate file references — no deduplication.
- Moving or renaming files on disk breaks playlist references (no library to reconcile).
- Large playlists (10,000+ tracks) may have slower load times compared to a database query.
- M3U8 export is lossy — mix points and extended metadata are not preserved.

**Neutral/Mitigations:**
- A future "Resolve Missing Files" feature could scan directories to relink broken paths.
- JSON playlist format is schema-versioned to allow forward-compatible evolution.
- Consider a virtual "All Tracks" playlist if users request cross-playlist browsing.
