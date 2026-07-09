## Why

The current progress bar shows only a filled bar with no time information, forcing users to guess how far into a track they are or how much time remains. Adding position, duration, and remaining-time labels transforms it into a proper playback indicator.

## What Changes

- Add a left label showing the current playback position (mm:ss or hh:mm:ss)
- Add a right label showing remaining time (mm:ss or hh:mm:ss)
- Add the total duration centered on the progress bar, with text color adapting to contrast against the progress fill
- Existing progress bar, click-to-seek, mix markers, and player-status event handling remain unchanged

## Capabilities

### New Capabilities
<!-- No new specs needed — this is a pure UI enhancement on existing data -->

No new capabilities. The change only modifies the frontend rendering of existing data (`position_secs`, `duration_secs` from the `player-status` event).

### Modified Capabilities
<!-- No existing spec-level behavior changes -->

None. The `playback-position` spec's requirements are unchanged.

## Impact

- **Frontend**: `src/main.ts` — add time label elements to the timeline container and update them in the `player-status` listener
- **Styles**: `src/styles.css` — add styles for the three new labels (position, duration, remaining) and the adaptive text color
- **No backend, API, or dependency changes**
