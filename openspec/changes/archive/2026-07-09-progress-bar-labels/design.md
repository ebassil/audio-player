## Context

The timeline container already exists in `src/main.ts` with a progress bar, mix-out/mix-in markers, and click-to-seek. The `player-status` event fires frequently (every ~50ms) delivering `position_secs`, `duration_secs`, and `progress` (0-1). Currently only `progress` is used to set the bar width.

## Goals / Non-Goals

**Goals:**
- Show current playback position as a time label left of the progress bar
- Show remaining time as a time label right of the progress bar
- Show total duration centered on the bar with text color that adapts to contrast against the progress fill
- Format: mm:ss for tracks under 1 hour, hh:mm:ss for tracks 1 hour+
- Work with the existing click-to-seek behavior

**Non-Goals:**
- Changing the backend `player-status` event payload
- Adding new Tauri commands or Rust code
- Changing playback position tracking logic
- Draggable timeline (out of scope)

## Decisions

1. **CSS `mix-blend-mode: difference` for adaptive duration text** — Instead of computing color inversions in JS on every frame, use `mix-blend-mode: difference` with a fixed white text color. The text automatically inverts against dark/light backgrounds. Layered inside the progress bar (clipped to the filled region) and outside (in the unfilled region), the two copies naturally create the correct contrast. Simpler than canvas or JS color math.

2. **Two `<span>` elements for duration, not one** — The centered duration needs to be visible across both the filled and unfilled portions of the bar. Two overlapping spans (one clipped to the progress fill, one in the unfilled area) with `mix-blend-mode: difference` achieve this without JS. Alternative considered: a single element with `-webkit-background-clip: text` — rejected as overly complex and less compatible.

3. **Position/remaining as separate elements outside the bar** — Placing them as flex items around the `.timeline` container keeps markup simple and avoids positioning conflicts with mix markers.

4. **No debouncing** — The `player-status` event already fires at a reasonable rate. Direct DOM updates for three text nodes are negligible.

## Risks / Trade-offs

- [`mix-blend-mode` browser support] → `mix-blend-mode: difference` is widely supported (Chrome/FF/Safari). Falls back to white text on non-supporting browsers (acceptable).
- [Flicker during rapid updates] → Small risk. Mitigated by using `requestAnimationFrame` or relying on the event rate being < 60fps.
- [Click-to-seek may overlap with labels] → Labels are positioned outside the `.timeline` bar area; click handler uses `e.clientX` on the bar element directly, so no conflict.

## Open Questions

- Should the remaining time show negative values when playback slightly exceeds duration? (e.g., if track metadata reports 300s but audio is 301s) — Decision: clamp to 0 and hide negative.
