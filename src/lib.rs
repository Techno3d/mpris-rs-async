//! # mpris-async
//!
//! Async version of the mpris crate. 
//!
//! Provides async versions of [`PlayerEvents`], [`PlayerFinder`],
//! and [`ProgressTracker`].
//!
//! # Get started 
//! Easiest way to get started with mpris is using [`get_active_player`] and then using
//! [`events::PlayerEventsStream`] to track changes.

use std::time::Duration;
use crate::player::PlayerStream;

use async_std::task;

pub use mpris::{Player, PlayerFinder, PlayerEvents, ProgressTracker, Progress, TrackList, TrackID};
use mpris::{DBusError, FindingError};

pub mod player;
pub mod events;
pub mod progress;
pub mod fake_progress;

/// Gets the most active player. If no player exists, this function will wait until one does.
/// Based of off [`PlayerFinder::find_active`](PlayerFinder::find_active)
pub async fn get_active_player(retry_delay: u64) -> Result<Player, DBusError> {
    let finder = match PlayerFinder::new() {
        Ok(x) => x,
        Err(_) => return  Err(DBusError::Miscellaneous("Could not create player finder. Is DBus running?".to_string())),
    };
    loop {
        let player = match finder.find_active() {
            Ok(player) => player,
            Err(FindingError::NoPlayerFound) => {
                task::sleep(Duration::from_millis(retry_delay)).await;
                continue
            },
            Err(FindingError::DBusError(x)) => return Err(x),
        };
        return Ok(player);
    }
}

/// Gets the first player. If no player exists, this function will wait until one does.
/// Based of off [`PlayerFinder::find_first`](PlayerFinder::find_first)
pub async fn get_first_player(retry_delay: u64) -> Result<Player, DBusError> {
    let finder = match PlayerFinder::new() {
        Ok(x) => x,
        Err(_) => return  Err(DBusError::Miscellaneous("Could not create player finder. Is DBus running?".to_string())),
    };
    loop {
        let player = match finder.find_first() {
            Ok(player) => player,
            Err(FindingError::NoPlayerFound) => {
                task::sleep(Duration::from_millis(retry_delay)).await;
                continue
            },
            Err(FindingError::DBusError(x)) => return Err(x),
        };
        return Ok(player);
    }
}

/// Gets all of the avaliable players. If no player exists, this function will wait until one does.
/// Every `retry_delay` milliseconds it will try for a new connection.
/// Based of off [`PlayerFinder::find_all`](PlayerFinder::find_all)
pub async fn get_players(retry_delay: u64) -> Result<Vec<Player>, DBusError> {
    let finder = match PlayerFinder::new() {
        Ok(x) => x,
        Err(_) => return  Err(DBusError::Miscellaneous("Could not create player finder. Is DBus running?".to_string())),
    };
    loop {
        let player = match finder.find_all() {
            Ok(player) => player,
            Err(FindingError::NoPlayerFound) => {
                task::sleep(Duration::from_millis(retry_delay)).await;
                continue
            },
            Err(FindingError::DBusError(x)) => return Err(x),
        };
        return Ok(player);
    }
}

/// Creates a stream of Players. Unlike mpris::PlayerFinder::iter_players, this function will keep
/// checking for more players forever.
pub fn stream_players(retry_delay: u64) -> PlayerStream {
    return PlayerStream::new(retry_delay);
}
