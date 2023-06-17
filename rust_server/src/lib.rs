/*

Build: manages the TCP connection
Listen: accepts new clients and adds them to the hashmap
    On an event, searches for the appropriate handler. Passes the event to them.
Events: Handlers for pre-defined routes.
        Provide a client handler for the event

*/

pub mod ws_io {

    pub enum To {
        Origin,
        NonOrigin,
        All,
    }

    use std::{
        collections::HashMap,
        net::SocketAddr,
        sync::{Arc, Mutex},
    };

    use futures_channel::mpsc::{unbounded, TrySendError, UnboundedSender};
    use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};

    use tokio::net::{TcpListener, TcpStream};
    use tokio_tungstenite::tungstenite::protocol::Message;

    type Tx = UnboundedSender<Message>;
    pub type ClientMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

    pub struct Socket<'a> {
        address: SocketAddr,
        clients: &'a ClientMap,
    }

    impl Socket<'_> {
        pub fn new(clients: &ClientMap, address: SocketAddr) -> Socket {
            Socket { address, clients }
        }

        pub fn send(&self, msg: Message, to: To) -> () {
            let clients = self.clients.lock().unwrap();
            match to {
                To::Origin => clients
                    .get(&self.address)
                    .unwrap()
                    .unbounded_send(msg)
                    .unwrap(),
                To::NonOrigin => clients
                    .iter()
                    .filter(|(&client_addr, _)| client_addr != self.address)
                    .for_each(|(_, ws_out)| ws_out.unbounded_send(msg.clone()).unwrap()),
                To::All => clients
                    .iter()
                    .for_each(|(_, ws_out)| ws_out.unbounded_send(msg.clone()).unwrap()),
            }
        }
    }

    pub struct Io {
        pub listener: TcpListener,
        pub clients: ClientMap,
    }

    impl Io {
        pub async fn build(address: &str) -> Result<Io, std::io::Error> {
            let listener = TcpListener::bind(&address).await?;
            println!("Listening on {}", address);
            Ok(Io {
                listener,
                clients: ClientMap::new(Mutex::new(HashMap::new())),
            })
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

            let socket = Socket::new(&client_map, addr);

            let catch_inbound = inbound.try_for_each(|msg| {
                socket.send(msg, To::All);
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
