use async_std::{task, stream::StreamExt};
use mpris_async::{stream_players, progress::ProgressStream, events::PlayerEventsStream};

// This is for testing to make sure that everything works like it is supposed to, which it isn't
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
            */
            let mut progress_stream = ProgressStream::new(&player, 1000).take(5);
            while let Some(progress) = progress_stream.next().await {
                println!("{}", progress.position().as_millis());
            }
        }
        
        //println!("{}", players.next().await.unwrap().identity());
    });
    println!("Hello, world!");
}
