use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// The possible states of the audio player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackState {
    /// No track loaded or playback completed.
    Stopped,
    /// Audio is actively playing.
    Playing,
    /// Playback is paused; position is preserved.
    Paused,
    /// Seeking to a new position; audio output is silent during seek.
    Seeking,
}

impl PlaybackState {
    pub fn description(&self) -> &'static str {
        match self {
            PlaybackState::Stopped => "No track loaded or playback completed",
            PlaybackState::Playing => "Audio is actively playing",
            PlaybackState::Paused => "Playback paused, position preserved",
            PlaybackState::Seeking => "Seeking to a new position",
        }
    }
}

/// Commands that can be sent to the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackCommand {
    Play,
    Pause,
    Resume,
    Stop,
    Seek,
    Next,
    Previous,
}

/// Manages playback state transitions.
///
/// Valid transitions:
/// - Stopped -> Playing
/// - Playing -> Paused
/// - Playing -> Stopped
/// - Playing -> Seeking -> Playing
/// - Paused -> Playing (resume)
/// - Paused -> Stopped
/// - Seeking -> Stopped (on error)
pub struct PlayerController {
    state: Arc<Mutex<PlaybackState>>,
}

impl PlayerController {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(PlaybackState::Stopped)),
        }
    }

    /// Attempt a state transition based on the given command.
    ///
    /// Returns `Ok(())` if the transition was valid, or `Err` with a message otherwise.
    pub fn send_command(&self, command: PlaybackCommand) -> Result<PlaybackState, String> {
        let mut state = self.state.lock().unwrap();
        let new_state = match (*state, command) {
            (PlaybackState::Stopped, PlaybackCommand::Play) => PlaybackState::Playing,
            (PlaybackState::Playing, PlaybackCommand::Pause) => PlaybackState::Paused,
            (PlaybackState::Playing, PlaybackCommand::Stop) => PlaybackState::Stopped,
            (PlaybackState::Playing, PlaybackCommand::Seek) => PlaybackState::Seeking,
            (PlaybackState::Paused, PlaybackCommand::Resume) => PlaybackState::Playing,
            (PlaybackState::Paused, PlaybackCommand::Play) => PlaybackState::Playing,
            (PlaybackState::Paused, PlaybackCommand::Stop) => PlaybackState::Stopped,
            (PlaybackState::Seeking, PlaybackCommand::Play) => PlaybackState::Playing,
            (PlaybackState::Seeking, PlaybackCommand::Stop) => PlaybackState::Stopped,
            (PlaybackState::Seeking, PlaybackCommand::Seek) => PlaybackState::Seeking,
            (_, PlaybackCommand::Next) | (_, PlaybackCommand::Previous) => {
                PlaybackState::Playing
            }
            (current, cmd) => {
                return Err(format!(
                    "Invalid transition: {:?} -> {:?}",
                    current, cmd
                ))
            }
        };
        *state = new_state;
        Ok(new_state)
    }

    /// Get the current state.
    pub fn state(&self) -> PlaybackState {
        *self.state.lock().unwrap()
    }

    /// Set the state directly (for internal use).
    pub fn set_state(&self, state: PlaybackState) {
        *self.state.lock().unwrap() = state;
    }

    /// Check if the player is in an active (playing or seeking) state.
    pub fn is_active(&self) -> bool {
        matches!(
            *self.state.lock().unwrap(),
            PlaybackState::Playing | PlaybackState::Seeking
        )
    }

    /// Check if playback is stopped.
    pub fn is_stopped(&self) -> bool {
        *self.state.lock().unwrap() == PlaybackState::Stopped
    }

    /// Check if playback is paused.
    pub fn is_paused(&self) -> bool {
        *self.state.lock().unwrap() == PlaybackState::Paused
    }
}

impl Default for PlayerController {
    fn default() -> Self {
        Self::new()
    }
}
