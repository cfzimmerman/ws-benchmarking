use my_ws::ws::{event::Event, ws_error::WsError, ws_io::Io};
use rust_server::redis_io::RedisConn;
use std::{env, sync::Arc};
// https://github.com/snapview/tokio-tungstenite/blob/master/examples/server.rs

#[tokio::main]
async fn main() -> Result<(), WsError> {
    let redis = Arc::new(tokio::sync::Mutex::new(
        RedisConn::build("redis://127.0.0.1/").await,
    ));

    let benchmark_event = rust_server::benchmark::handle_message(redis);
    let event_list: Vec<Event> = vec![Event {
        path: "benchmark",
        action: benchmark_event,
    }];

    let server_address = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:5445".to_string());

    let io = Io::build(&server_address, event_list).await?;
    io.listen().await;

    Ok(())
}

/*

Next up:
- Write a closure to handle the benchmark path
    - Abstract stuff back into lib
- Read back the hash
- Send the hash back as a response

*/
