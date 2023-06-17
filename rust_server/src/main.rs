use rust_server::ws_io;
use std::env;

// https://github.com/snapview/tokio-tungstenite/blob/master/examples/server.rs

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let address = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:5445".to_string());

    let io = ws_io::Io::build(&address).await?;
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
