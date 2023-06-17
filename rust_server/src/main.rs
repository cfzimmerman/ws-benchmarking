use my_ws::ws::{
    event::{Event, EventAction},
    socket::To,
    ws_error::WsError,
    ws_io::Io,
};
use std::{boxed::Box, env};
use tokio_tungstenite::tungstenite::Message;

// https://github.com/snapview/tokio-tungstenite/blob/master/examples/server.rs

#[tokio::main]
async fn main() -> Result<(), WsError> {
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
