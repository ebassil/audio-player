## ADR-004: Global Shortcut Architecture

* **Status:** Accepted
* **Date:** 2026-07-07
* **Author:** Architect

### Context

The audio player must support keyboard shortcuts that work even when the application window is not focused, allowing users to control playback while working in other applications. Shortcuts must be user-configurable via both a config file and a UI settings panel. Some actions (like DeletePlus with confirmation) require session-level toggles that reset on application restart.

macOS imposes restrictions on global hotkeys — certain key combinations are reserved by the system (Cmd+Tab, Cmd+Q, Cmd+Space, etc.) and cannot be overridden. Tauri's `global-shortcut` plugin uses Carbon's `RegisterEventHotKey` under the hood, which has well-known limitations.

### Decision

We will use `tauri-plugin-global-shortcut` for registering global hotkeys. Shortcut definitions will be stored in a config file (TOML) and editable from a UI settings panel. Multi-modifier chords (e.g., Ctrl+Alt+Shift+Key) will be used by default to minimize OS conflicts. A conflict detection mechanism will validate bindings at registration time and surface failures in the UI. Session-level toggles (like "disable delete confirmation") will be stored in memory only and reset to defaults on application restart.

### Visual Architecture

```mermaid
graph TD
    subgraph Configuration
        A[shortcuts.toml] --> B{<br/>play = "Ctrl+Alt+P"<br/>next = "Ctrl+Alt+Right"<br/>prev = "Ctrl+Alt+Left"<br/>delete = "Delete"<br/>delete_plus = "Shift+Delete"<br/>vol_up = "Ctrl+Alt+Up"<br/>vol_down = "Ctrl+Alt+Down"<br/>}
        C[Session State] --> D{<br/>confirm_delete: true<br/>}
    end

    subgraph Rust Backend
        E[Shortcut Engine] -->|Read bindings| A
        E -->|Register| F[tauri-plugin-<br/>global-shortcut]
        F -->|RegisterEventHotKey| G[macOS Carbon API]
        F -->|on_shortcut| H[Dispatch Action]
        H -->|play| I[Audio Pipeline]
        H -->|delete_plus| J[Delete Handler]
        J -->|check| C
        C -->|confirm: true| K[Show Dialog]
        C -->|confirm: false| L[Delete Directly]
        M[Conflict Detector] -->|validate| F
        M -->|conflict| N[Log + UI Notification]
    end

    subgraph Webview UI
        O[Settings Panel] -->|Read/Write| A
        O -->|Rebind| E
        P[Delete Dialog] -->|"Don't ask again"| C
    end
```

### Consequences

**Positive (Benefits):**
- Shortcuts work globally, even when the app is backgrounded.
- Config file is human-editable for power users.
- Session toggles give users fast workflows without permanent risk.
- Conflict detection prevents silent shortcut failures.

**Negative (Risks/Trade-offs):**
- macOS limits the pool of available chords — users may need to experiment to find non-conflicting combinations.
- Some keys may register on one macOS version but conflict on another due to OS updates.
- Session toggles are ephemeral — a crash or force-quit loses the toggle state.
- `tauri-plugin-global-shortcut` may have platform-specific edge cases on macOS (e.g., certain accessibility permissions required).

**Neutral/Mitigations:**
- Provide a "Reset to Defaults" button in the UI for shortcut configuration.
- Document known-conflicting macOS shortcuts in the application help.
- Persist session toggle state to a temp file as a best-effort recovery mechanism.
- Test global shortcuts with macOS Screen Recording / Accessibility permissions flow.
