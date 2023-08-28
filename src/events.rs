//! [`PlayerEventsStream`] handles when new player events are emitted by a given player.
//! Alternatively, the class gives a reciever which can be used to track events.

use std::{task::{Waker, Poll}, thread};

use async_std::{channel::{Sender, Receiver, unbounded, TryRecvError}, task, stream::Stream};
use mpris::{Player, Event, PlayerFinder};

/// Infinite Stream which tracks the Events emitted by a player. Streams created with new create a
/// new thread to track events. Use clone instead of new when trying to track changes on the same
/// player to avoid creating multiple threads that do the same work.
#[derive(Debug, Clone)]
pub struct PlayerEventsStream {
    identity: String,
    sender: Sender<Event>,
    reciever: Receiver<Event>,
    wakers: (Sender<Waker>, Receiver<Waker>),
}

impl PlayerEventsStream {
    /// Creates a new [`PlayerEventsStream`] to track the changes of a player. This function will
    /// spawn a new thread that listens for changes and sends them to the stream and any stream
    /// cloned from it.
    pub fn new(player: &Player) -> PlayerEventsStream {
        let (s, r) = unbounded();
        let (wake_send, wake_reciev) = unbounded();
        let streamer = PlayerEventsStream {identity: player.identity().to_string(), sender: s, reciever: r, wakers: (wake_send, wake_reciev)};
        let stream_clone = streamer.clone();
        thread::spawn(move || stream_clone.events_listener());
        return streamer;
    }

    fn events_listener(&self) {
        let finder = match PlayerFinder::new() {
            Ok(x) => x,
            Err(_) => {
                self.sender.try_send(Event::PlayerShutDown).unwrap();
                self.sender.close();
                return;
            },
        };
        let player = finder.find_by_name(&self.identity).unwrap();
        let events = player.events().unwrap();
        
        for event in events {
            self.sender.try_send(match event.as_ref().unwrap() {
                Event::PlayerShutDown => Event::PlayerShutDown,
                Event::Paused => Event::Paused,
                Event::Playing => Event::Playing,
                Event::Stopped => Event::Stopped,
                Event::LoopingChanged(status) => Event::LoopingChanged(*status),
                Event::ShuffleToggled(x) => Event::ShuffleToggled(*x),
                Event::VolumeChanged(x) => Event::VolumeChanged(*x),
                Event::PlaybackRateChanged(x) => Event::PlaybackRateChanged(*x),
                Event::TrackChanged(x) => Event::TrackChanged(x.clone()),
                Event::Seeked { position_in_us } => Event::Seeked { position_in_us: *position_in_us },
                Event::TrackAdded(x) => Event::TrackAdded(x.clone()),
                Event::TrackRemoved(x) => Event::TrackRemoved(x.clone()),
                Event::TrackMetadataChanged { old_id, new_id } => Event::TrackMetadataChanged { old_id: old_id.clone(), new_id: new_id.clone() },
                Event::TrackListReplaced => Event::TrackListReplaced,
            }).unwrap();
            if matches!(event.unwrap(), Event::PlayerShutDown) {
                self.sender.close();
            }

            loop {
                match self.wakers.1.try_recv() {
                    Ok(waker) => {
                        waker.wake_by_ref();
                    }
                    _ => break,
                }
            }
        }
    }

    /// Access to the reciever used to send Events around.
    pub fn get_reciever(&self) -> Receiver<Event> {
        self.reciever.clone()
    }
}

impl Stream for PlayerEventsStream {
    type Item = Event;

    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Option<Self::Item>> {
        let reciever = self.reciever.clone();
        self.wakers.0.try_send(cx.waker().clone()).unwrap();
        let event = reciever.try_recv();
        match event {
            Ok(event) => {
                cx.waker().wake_by_ref();
                Poll::Ready(Some(event))
            },
            Err(TryRecvError::Empty) => Poll::Pending,
            Err(TryRecvError::Closed) => Poll::Ready(None),
        }
    }
}
