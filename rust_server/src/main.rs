use my_ws::ws::{event::Event, ws_error::WsError, ws_io::Io};
use rust_server::{
    benchmark::{Counters, Payload},
    redis_io::RedisConn,
};
use std::env;
// https://github.com/snapview/tokio-tungstenite/blob/master/examples/server.rs

#[tokio::main]
async fn main() -> Result<(), WsError> {
    let mut rd = RedisConn::build("redis://127.0.0.1/").await;
    let test_counters = Counters { current: 8, to: 7 };
    let id = "rust";
    let test_payload = Payload {
        id: id.to_string(),
        counters: test_counters,
    };
    rd.set_benchmark_payload(test_payload).await;
    let res = rd.get_benchmark_payload(Payload::redis_key(id)).await;
    println!("redis res: {:?}", res);

    let address = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:5445".to_string());

    /*
    let echo: EventAction = Box::new(|socket, message| {
        if let Err(error) = socket.send(Message::Text(message), To::All) {
            eprintln!("event failed: {:?}", error)
        };
    });
    */

    let event_list: Vec<Event> = vec![];

    let io = Io::build(&address, event_list).await?;
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
