## ADDED Requirements

### Requirement: Configurable Shortcut Bindings
The system SHALL provide a default set of keyboard shortcuts: Play/Pause, Next Track, Previous Track, Delete, DeletePlus, Volume Up, Volume Down, Mute, and Seek Forward/Backward.
Shortcuts SHALL be configurable via a config file (JSON or TOML) and editable from the UI.
The system SHALL validate that shortcut bindings do not conflict with each other and notify the user of conflicts in the UI.

#### Scenario: Default shortcuts active
- **WHEN** the application starts for the first time
- **THEN** default shortcuts SHALL be active

#### Scenario: Rebind shortcut from UI
- **WHEN** the user rebinds "Play/Pause" from Space to Ctrl+P in the settings UI
- **THEN** pressing Ctrl+P SHALL toggle playback and Space SHALL no longer do so

#### Scenario: Conflicting shortcut warning
- **WHEN** the user assigns the same key combination to two actions
- **THEN** the UI SHALL display a conflict warning

### Requirement: Global Shortcuts (Tauri v2)
The system SHALL register configured shortcuts as global hotkeys using `tauri-plugin-global-shortcut` (v2 npm/Cargo package). The capability `global-shortcut:default` SHALL be declared in `src-tauri/capabilities/default.json` per ADR-006.
Global shortcuts SHALL work when the application window is not in focus (user is working in other applications).
Shortcuts SHALL use unique multi-modifier chords (e.g., Ctrl+Alt+Shift+Key) to minimize conflicts with OS-level shortcuts.

#### Scenario: Global shortcut triggers playback
- **WHEN** the user presses the global Play/Pause shortcut while another app is focused
- **THEN** the audio player SHALL toggle playback

#### Scenario: Unregisterable shortcut
- **WHEN** a shortcut conflicts with a system-level shortcut and cannot be registered
- **THEN** the system SHALL log the failure and notify the user in the settings UI

### Requirement: Session-Level Toggles
The system SHALL support boolean toggles that can be enabled/disabled for the duration of a session.
The "Confirm Delete" toggle SHALL be available: when enabled, DeletePlus shows a confirmation dialog; when disabled, DeletePlus deletes without confirmation.
Session toggles SHALL reset to their default state when the application is restarted.

#### Scenario: Disable delete confirmation for session
- **WHEN** the user unchecks "Confirm delete" in the DeletePlus dialog
- **THEN** subsequent DeletePlus actions SHALL skip confirmation for this session

#### Scenario: Session toggle resets on restart
- **WHEN** the user restarts the application
- **THEN** the "Confirm delete" toggle SHALL be enabled again by default

### Requirement: Shortcut Actions Extensibility
The shortcut system SHALL support adding new actions without modifying the core shortcut engine (future actions: set song mood, like/dislike toggle, etc.).
New actions SHALL be defined by adding entries to the shortcuts config file with an action name and key binding.

#### Scenario: Add new shortcut action
- **WHEN** a future update adds a "like_toggle" action
- **THEN** it SHALL be bindable through the existing shortcut configuration UI without engine changes
