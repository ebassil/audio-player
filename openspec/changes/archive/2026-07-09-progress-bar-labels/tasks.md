## 1. HTML Markup: Add timeline labels to src/main.ts

- [x] 1.1 Add `<span id="position-label" class="time-label time-label--position"></span>` before the `.timeline` div
- [x] 1.2 Add `<span id="duration-label-fill" class="time-label time-label--duration"></span>` inside the `.progress-bar` element
- [x] 1.3 Add `<span id="duration-label-bg" class="time-label time-label--duration"></span>` directly after `#progress-bar` (same parent as bar, overlays the unfilled area)
- [x] 1.4 Add `<span id="remaining-label" class="time-label time-label--remaining"></span>` after the `.timeline` div
- [x] 1.5 Wrap the timeline row in a flex container with the new labels

## 2. CSS: Style labels in src/styles.css

- [x] 2.1 Style `.time-label` with font-size, font-family (monospace), color (#666 for position/remaining)
- [x] 2.2 Style `.time-label--position` left-aligned
- [x] 2.3 Style `.time-label--remaining` right-aligned
- [x] 2.4 Style `.time-label--duration` with white text, `mix-blend-mode: difference`, absolute centered positioning, pointer-events: none
- [x] 2.5 Style the flex container `.timeline-row` to align position/bar/remaining in a row
- [x] 2.6 Clip `#duration-label-fill` to the progress bar width so it only renders over the filled region
- [x] 2.7 Ensure `#duration-label-bg` renders over the unfilled region (outside the progress bar)

## 3. JS: Format and update time labels in src/main.ts

- [x] 3.1 Create a helper `formatTime(secs: number): string` that returns `mm:ss` if secs < 3600, else `hh:mm:ss`, clamping negative values to `0:00`
- [x] 3.2 In the `player-status` listener, compute `remaining = max(0, duration_secs - position_secs)`
- [x] 3.3 Update `#position-label` textContent with `formatTime(position_secs)`
- [x] 3.4 Update `#remaining-label` textContent with `formatTime(remaining)`
- [x] 3.5 Update both `#duration-label-fill` and `#duration-label-bg` textContent with `formatTime(duration_secs)`
- [x] 3.6 Set `#duration-label-fill`'s `clip-path` (or equivalent) to match the current progress percentage so it only shows over the filled area
