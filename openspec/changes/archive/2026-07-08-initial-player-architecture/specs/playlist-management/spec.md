## ADDED Requirements

### Requirement: Playlist JSON Format
The primary playlist format SHALL be `.json` containing: an array of track entries, each with file path, mix-in/mix-out points (optional), mix pattern override (optional), and any per-song metadata.
The JSON playlist file SHALL be self-contained and usable without a companion `.m3u8` file.

#### Scenario: Save playlist as JSON
- **WHEN** the user saves a playlist
- **THEN** the system SHALL write a JSON file with all track data including mix points and metadata

#### Scenario: Load playlist from JSON
- **WHEN** the user opens a .json playlist file
- **THEN** the system SHALL load all tracks and restore mix points and metadata

### Requirement: M3U8 Playlist Format
The system SHALL support exporting and importing `.m3u8` playlist files for cross-player compatibility.
The M3U8 format SHALL contain file paths only — extended features (mix points, etc.) are not stored in M3U8.

#### Scenario: Export to M3U8
- **WHEN** the user exports a playlist as M3U8
- **THEN** the system SHALL write an M3U8 file with the file paths of all tracks

#### Scenario: Import from M3U8
- **WHEN** the user opens an M3U8 file
- **THEN** the system SHALL load all referenced tracks (mix points will be empty for all)

### Requirement: Track Import — Drag and Drop
The system SHALL allow the user to drag and drop audio files or folders from the file system into the application.
Dropped files SHALL populate a new playlist in-memory. The user can then save this as a playlist file.

#### Scenario: Drag-drop audio files
- **WHEN** the user drags audio files into the application window
- **THEN** those files SHALL appear as tracks in a new playlist

#### Scenario: Drag-drop a folder
- **WHEN** the user drags a folder into the application window
- **THEN** all supported audio files in that folder and its subdirectories SHALL be added as tracks

### Requirement: Track Import — Directory Loading
The system SHALL provide a UI action to load all supported audio files from a selected directory and its subdirectories.
The loaded files SHALL populate a new playlist in-memory.

#### Scenario: Load directory from UI
- **WHEN** the user selects "Load Directory" from the UI and chooses a folder
- **THEN** all supported audio files recursively found in that folder SHALL populate a new playlist

### Requirement: Track Removal
Delete SHALL remove the selected track(s) from the current playlist only. No confirmation dialog SHALL be shown.
DeletePlus SHALL remove the selected track(s) from the current playlist AND delete the file from disk. A confirmation dialog SHALL be shown with a "Don't ask again this session" checkbox.
Deleted tracks SHALL be removed from the playlist JSON when saved.

#### Scenario: Delete track from playlist
- **WHEN** the user presses Delete on a selected track
- **THEN** the track SHALL be removed from the playlist

#### Scenario: DeletePlus with confirmation
- **WHEN** the user presses DeletePlus on a selected track
- **THEN** a confirmation dialog SHALL appear
- **WHEN** the user confirms
- **THEN** the track SHALL be removed from the playlist AND the file SHALL be deleted from disk

#### Scenario: DeletePlus without confirmation (session toggle)
- **WHEN** the user checks "Don't ask again this session" in the DeletePlus confirmation
- **THEN** subsequent DeletePlus actions in the same session SHALL skip the confirmation dialog
- **WHEN** the application is restarted
- **THEN** the confirmation dialog SHALL reappear
