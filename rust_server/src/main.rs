use my_ws::ws::{
    event::{Event, EventAction},
    socket::To,
    ws_error::WsError,
    ws_io::{Io, Message},
};
use rust_server::{
    benchmark::{Counters, Payload},
    redis_io::RedisConn,
};
use std::{boxed::Box, env};
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

    let echo: EventAction = Box::new(|socket, message| {
        if let Err(error) = socket.send(Message::Text(message), To::All) {
            eprintln!("event failed: {:?}", error)
        };
    });

    let event_list: Vec<Event> = vec![Event::new("echo", echo)];

    let io = Io::build(&address, event_list).await?;
    io.listen().await;

    Ok(())
}

/*

To do:
- Mount WS listeners

Next up:
- Configure routes
    - path: string
    - destination?
    - method


- Connect to Redis
- Receive data, parse it:
    count: number,
    origin: string,
- Update a hash in redis
- Read back the hash
- Send the hash back as a response

*/
