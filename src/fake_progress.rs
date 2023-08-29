//! Used to make [`crate::progress::ProgressStream`] possible. The code is identical to mpris's
//! Progress.
use std::time::{Instant, Duration};

use mpris::{LoopStatus, PlaybackStatus, Metadata, Progress};

/// Used Because Cloning Progress is impossible, making [`crate::progress::ProgressStream`]
/// impossible for me to implement
#[derive(Clone)]
pub struct FakeProgress {
    pub(crate) metadata: Metadata,
    pub(crate) playback_status: PlaybackStatus,
    pub(crate) shuffle: bool,
    pub(crate) loop_status: LoopStatus,

    /// When this Progress was constructed, in order to calculate how old it is.
    pub(crate) instant: Instant,

    pub(crate) position: Duration,
    pub(crate) rate: f64,
    pub(crate) current_volume: f64,
}

impl FakeProgress {
    pub(crate) fn from(progress: &Progress) -> Self {
        FakeProgress {
            metadata: progress.metadata().clone(),
            playback_status: progress.playback_status(),
            shuffle: progress.shuffle(),
            loop_status: progress.loop_status(),
            instant: *progress.created_at(),
            position: progress.position(),
            rate: progress.playback_rate(),
            current_volume: progress.current_volume(),
        }
    }
    /// The track metadata at the point in time that this Progress was constructed.
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// The playback status at the point in time that this Progress was constructed.
    pub fn playback_status(&self) -> PlaybackStatus {
        self.playback_status
    }

    /// The shuffle status at the point in time that this Progress was constructed.
    pub fn shuffle(&self) -> bool {
        self.shuffle
    }

    /// The loop status at the point in time that this Progress was constructed.
    pub fn loop_status(&self) -> LoopStatus {
        self.loop_status
    }

    /// The playback rate at the point in time that this Progress was constructed.
    pub fn playback_rate(&self) -> f64 {
        self.rate
    }

    /// Returns the length of the current track as a [`Duration`].
    pub fn length(&self) -> Option<Duration> {
        self.metadata.length()
    }

    /// Returns the current position of the current track as a [`Duration`].
    ///
    /// This method will calculate the expected position of the track at the instant of the
    /// invocation using the [`initial_position`](Self::initial_position) and knowledge of how long ago that position was
    /// determined.
    ///
    /// **Note:** Some players might not support this and will return a bad position. Spotify is
    /// one such example. There is no reliable way of detecting problematic players, so it will be
    /// up to your client to check for this.
    ///
    /// One way of doing this is to query the [`initial_position`](Self::initial_position) for two measures with the
    /// [`PlaybackStatus::Playing`] and if both are `0`, then it is likely that this client does not
    /// support positions.
    pub fn position(&self) -> Duration {
        self.position + self.elapsed()
    }

    /// Returns the position that the current track was at when the [`Progress`] was created.
    ///
    /// This is the number that was returned for the [`Position`][position] property in the MPRIS2 interface.
    ///
    /// [position]: https://specifications.freedesktop.org/mpris-spec/latest/Player_Interface.html#Property:Position
    pub fn initial_position(&self) -> Duration {
        self.position
    }

    /// The instant where this [`Progress`] was recorded.
    ///
    /// See: [`age`](Self::age).
    pub fn created_at(&self) -> &Instant {
        &self.instant
    }

    /// Returns the age of the data as a [`Duration`].
    ///
    /// If the [`Progress`] has a high age it is more likely to be out of date.
    pub fn age(&self) -> Duration {
        self.instant.elapsed()
    }

    /// Returns the player's volume as it was at the time of refresh.
    ///
    /// See: [`Player::get_volume`].
    pub fn current_volume(&self) -> f64 {
        self.current_volume
    }

    fn elapsed(&self) -> Duration {
        let elapsed_ms = match self.playback_status {
            PlaybackStatus::Playing => {
                DurationExtensions::as_millis(&self.age()) as f64 * self.rate
            }
            _ => 0.0,
        };
        Duration::from_millis(elapsed_ms as u64)
    }
}


pub(crate) trait DurationExtensions {
    // Rust beta has a from_micros function that is unstable.
    fn from_micros_ext(_: u64) -> Duration;
    fn as_millis(&self) -> u64;
    fn as_micros(&self) -> u64;
}

impl DurationExtensions for Duration {
    fn from_micros_ext(micros: u64) -> Duration {
        let whole_seconds = micros / 1_000_000;
        let rest = (micros - (whole_seconds * 1_000_000)) as u32;
        Duration::new(whole_seconds, rest * 1000)
    }

    fn as_millis(&self) -> u64 {
        self.as_secs() * 1000 + u64::from(self.subsec_millis())
    }

    fn as_micros(&self) -> u64 {
        self.as_secs() * 1000 * 1000 + u64::from(self.subsec_micros())
    }
}
