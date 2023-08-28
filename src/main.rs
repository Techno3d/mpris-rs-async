use async_std::{task, stream::StreamExt};
use mpris_async::{stream_players, events::PlayerEventsStream};

fn main() {
    task::block_on(async {
        let mut players = stream_players(100);
        //while let Some(player) = players.next().await {
        //     println!("{}", player.identity());
        //}
        
        let player = players.next().await.unwrap();
        println!("{}", player.identity());
        let mut events_stream = PlayerEventsStream::new(&player).take(5);
        while let Some(event) = events_stream.next().await {
            println!("{:?}", event);
        }
        //println!("{}", players.next().await.unwrap().identity());
        println!("AA");
    });
    println!("Hello, world!");
}
