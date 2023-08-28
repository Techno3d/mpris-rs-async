use std::{task::{Waker, Poll}, thread};

use async_std::{channel::{unbounded, Sender, Receiver}, stream::Stream};
use mpris::{Player, PlayerFinder};
pub use mpris::ProgressTracker;

use crate::fake_progress::FakeProgress;


/// Streams changes from [`ProgressTracker`]. Makes a new thread to track changes from the player.
#[derive(Debug, Clone)]
pub struct ProgressStream {
    // Cannot share Player, Progress Tick, and TrackList across thread/tasks
    identity: String,
    waker: (Sender<Waker>, Receiver<Waker>),
    interval: u32,
}

impl ProgressStream {
    /// Creates a new [`ProgressStream`] and a new thread to track changes. All ProgressStreams
    /// made from cloning will use the same thread to track changes. The thread only closes when
    /// the player has quit.
    pub fn new(player: &Player, interval: u32) -> Self {
       let waker = unbounded();
       let streamer = ProgressStream {identity: player.identity().to_string(), waker, interval };
       let stream_clone = streamer.clone();
       thread::spawn(|| stream_clone.progress_listener());

       return streamer;
    }

    fn progress_listener(self) {
        let finder = match PlayerFinder::new() {
            Ok(x) => x,
            Err(_) => {
                return;
            },
        };
        let player = finder.find_by_name(&self.identity).unwrap();
        let test = player.track_progress(self.interval);
        let mut progress_tracker = match test {
            Ok(x) => x,
            Err(e) => panic!("{}", e),
        };
        loop {
            let tick = progress_tracker.tick();
            if tick.player_quit {
                return;
            }
            loop {
                match self.waker.1.try_recv() {
                    Ok(waker) => {
                        if tick.progress_changed {
                            waker.wake_by_ref();
                        }
                    }
                    _ => break,
                }
            }

            
        }
    }
}

impl Stream for ProgressStream {
    type Item = FakeProgress;

    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        let finder = match PlayerFinder::new() {
            Ok(x) => x,
            Err(_) => {
                return Poll::Ready(None);
            },
        };
        let player = finder.find_by_name(&self.identity).unwrap();
        let mut tracker = player.track_progress(1).unwrap();
        let tick = tracker.tick();
        let progress = tick.progress;
        if tick.progress_changed {
            return Poll::Ready(Some(FakeProgress {
                metadata: progress.metadata().clone(),
                playback_status: progress.playback_status(),
                shuffle: progress.shuffle(),
                loop_status: progress.loop_status(),
                instant: *progress.created_at(),
                position: progress.position(),
                rate: progress.playback_rate(),
                current_volume: progress.current_volume(),
            }));
        }
        match self.waker.0.try_send(cx.waker().clone()) {
            Ok(_) => {},
            Err(_) => return Poll::Ready(None),
        };
        return Poll::Pending;
    }
}
