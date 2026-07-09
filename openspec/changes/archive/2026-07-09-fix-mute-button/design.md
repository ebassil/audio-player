## Context

The audio pipeline reads decoded PCM samples from `BufferedDecoder` in its output callback and sends them directly to the cpal output device — no gain/volume multiplication is applied. The `VolumeState` struct exists with `effective_gain()` returning `0.0` when muted, but it is never consulted in the audio callback.

The `VolumeNode` fundsp AudioNode exists (`volume.rs:76-105`) but is never inserted into the pipeline's `AudioGraph`.

## Goals / Non-Goals

**Goals:**
- Mute button audibly silences playback
- Unmute restores prior volume level
- Volume slider (if any) continues to work unchanged

**Non-Goals:**
- Rewiring the fundsp graph to use `VolumeNode` (that would be a larger refactor)
- Changing the `VolumeState` API
- Adding new UI elements

## Decisions

**Decision 1: Apply gain inline in the callback vs wire up VolumeNode**
- *Option A (chosen):* Add an `Arc<VolumeState>` capture to the callback closure and multiply output samples by `volume.effective_gain()`.
- *Option B:* Insert `VolumeNode` into the `AudioGraph` and route audio through it.
- *Rationale:* Option A is minimal, correct, and matches the existing pattern where the callback directly processes decoder output. The `VolumeState` is already `Send + Sync` (uses `Arc<Mutex>` and `Arc<AtomicBool>`). Option B would require restructuring how the graph processes audio (it currently only handles pre/post effects, not per-sample gain).

**Decision 2: Share VolumeState via Arc**
- `VolumeState` is owned by `AudioPipeline` (not behind `Arc`). The callback closure needs access to it. We'll add a new `Arc<VolumeState>` field or extract the mute flag and gain into atomic values that the callback can capture.
- *Rationale:* Rather than wrapping the whole `VolumeState` in a second `Arc`, the simplest approach is to capture an `Arc<AtomicBool>` for the mute flag and an `Arc<Mutex<f64>>` for the gain — these already exist inside `VolumeState`. But since `VolumeState` does not expose its internals as `Arc` references publicly, the cleanest approach is to store a shared `Arc<VolumeState>` on the pipeline for the callback to capture.

## Risks / Trade-offs

- **[Performance]** The callback runs on the audio thread (real-time). Multiplying each sample by gain is a simple float operation and already matches what `VolumeNode::tick` would do. Negligible overhead.
- **[Thread Safety]** `effective_gain()` locks an internal `Mutex` and reads an `AtomicBool`. The callback must not hold locks long; this is a brief lock/unlock per callback invocation — safe and fast.
- **[Duplicate state]** If the frontend also remembers mute state (button icon), the backend is source of truth. No conflict as long as `saveAppConfig` persists the backend state.