//! [`ProgressStream`] handles when changes to progress are sent.

use std::{task::{Waker, Poll}, thread};

use async_std::{channel::{unbounded, Sender, Receiver}, stream::Stream};
use mpris::{Player, PlayerFinder};

use crate::fake_progress::ProgressClone;


/// Streams changes from [`ProgressTracker`](mpris::ProgressTracker). Makes a new thread to track changes from the player.
/// This class will only send progress when it has changed since the last check.
#[derive(Debug, Clone)]
pub struct ProgressStream {
    // Cannot share Player, Progress Tick, and TrackList across thread/tasks
    identity: String,
    progress_channel: (Sender<MaybeProgress>, Receiver<MaybeProgress>),
    waker: (Sender<Waker>, Receiver<Waker>),
    interval: u32,
}

enum MaybeProgress {
    ProgressFake(ProgressClone), Stopped,
}

impl ProgressStream {
    /// Creates a new [`ProgressStream`] and a new thread to track changes. All ProgressStreams
    /// made from cloning will use the same thread to track changes. The thread only closes when
    /// the player has quit.
    pub fn new(player: &Player, interval: u32) -> Self {
       let waker = unbounded();
       let progress_channel = unbounded();
       let streamer = ProgressStream {identity: player.identity().to_string(), progress_channel, waker, interval };
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
        let player = match finder.find_by_name(&self.identity) {
            Ok(x) => x,
            Err(_) => {
                match self.progress_channel.0.try_send(MaybeProgress::Stopped) {
                    Ok(_) => {},
                    Err(_) => return,
                };
                return;
            },
        };
        let test = player.track_progress(self.interval);
        let mut progress_tracker = match test {
            Ok(x) => x,
            Err(e) => panic!("{}", e),
        };
        loop {
            let tick = progress_tracker.tick();
            if tick.player_quit {
                match self.progress_channel.0.try_send(MaybeProgress::Stopped) {
                    Ok(x) => x,
                    Err(_) => return,
                };

                return;
            }
            if tick.progress_changed {
                loop {
                    match self.waker.1.try_recv() {
                        Ok(waker) => {
                            match self.progress_channel.0.try_send(MaybeProgress::ProgressFake(ProgressClone::from(tick.progress))) {
                                Ok(_) => {},
                                Err(_) => return,
                            };
                            waker.wake_by_ref();
                        }
                        _ => break,
                    }
                }
            }

            
        }
    }
}

impl Stream for ProgressStream {
    type Item = ProgressClone;

    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        match self.waker.0.try_send(cx.waker().clone()) {
            Ok(_) => {},
            Err(_) => return Poll::Ready(None),
        };

        let progress = match self.progress_channel.1.try_recv() {
            Ok(x) => x,
            Err(_) => return Poll::Pending,
        };


        match progress {
            MaybeProgress::ProgressFake(x) => return Poll::Ready(Some(x)),
            MaybeProgress::Stopped => return Poll::Ready(None),
        }


    }
}
