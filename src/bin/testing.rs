use async_std::{task, stream::StreamExt};
use mpris_async::{stream_players, progress::ProgressStream, events::PlayerEventsStream};

fn main() {
    task::block_on(async {
        let mut players = stream_players(100);
        while let Some(player) = players.next().await {
            println!("{}", player.identity());
            let mut events_stream = PlayerEventsStream::new(&player).take(5);
            while let Some(event) = events_stream.next().await {
                println!("{:?}", event);
            }
            /*
                Code put in here for testing purposes
            */
            let mut progress_stream = ProgressStream::new(&player, 1000).take(5);
            while let Some(progress) = progress_stream.next().await {
                println!("{}", progress.position().as_millis());
            }
        }
    });
}
