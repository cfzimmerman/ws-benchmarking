use std::{
    collections::HashMap,
    env,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use futures_channel::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::protocol::Message;

// https://github.com/snapview/tokio-tungstenite/blob/master/examples/server.rs

type Tx = UnboundedSender<Message>;

pub type ClientMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

mod ws_io {
    use std::{
        collections::HashMap,
        net::SocketAddr,
        sync::{Arc, Mutex},
    };

    use futures_channel::mpsc::{unbounded, UnboundedSender};
    use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};

    use tokio::net::{TcpListener, TcpStream};
    use tokio_tungstenite::tungstenite::protocol::Message;

    type Tx = UnboundedSender<Message>;
    pub type ClientMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

    pub struct Io {
        pub listener: TcpListener,
        pub clients: ClientMap,
    }

    impl Io {
        pub async fn build(address: &str) -> Io {
            let try_socket = TcpListener::bind(&address).await;
            let listener = try_socket.expect("Failed to bind to socket");
            println!("Listening on {}", address);
            Io {
                listener,
                clients: ClientMap::new(Mutex::new(HashMap::new())),
            }
        }

        pub async fn listen(&self) -> () {
            while let Ok((stream, addr)) = self.listener.accept().await {
                tokio::spawn(Io::manage_connection(self.clients.clone(), stream, addr));
            }
        }

        async fn manage_connection(client_map: ClientMap, stream: TcpStream, addr: SocketAddr) {
            let ws_stream = tokio_tungstenite::accept_async(stream)
                .await
                .expect("Error during WS handshake");
            println!("WS connection established: {}", addr);

            let (sender, receiver) = unbounded();
            client_map.lock().unwrap().insert(addr, sender);

            let (outbound, inbound) = ws_stream.split();

            let catch_inbound = inbound.try_for_each(|msg| {
                let clients = client_map.lock().unwrap();

                let non_origin = clients
                    .iter()
                    .filter(|(client_addr, _)| client_addr != &&addr)
                    .map(|(_, ws_out)| ws_out);

                println!("{:?}", msg);
                for client in non_origin {
                    client.unbounded_send(msg.clone()).unwrap();
                }

                future::ok(())
            });

            let receive_from_others = receiver.map(Ok).forward(outbound);

            pin_mut!(catch_inbound, receive_from_others);
            future::select(catch_inbound, receive_from_others).await;

            println!("{} disconnected", &addr);
            client_map.lock().unwrap().remove(&addr);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let address = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:5445".to_string());

    let io = ws_io::Io::build(&address).await;
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
