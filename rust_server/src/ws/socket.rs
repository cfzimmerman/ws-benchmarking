use crate::ws::{ws_error::WsError, ws_io::ClientMap};
use std::net::SocketAddr;
use tokio_tungstenite::tungstenite::protocol::Message;

/// To: Who to send a WS message to.
/// Origin: Send to the client we just received the message from.
/// NonOrigin: Send to everyone except the client we just received the message from.
/// All: Send to everyone connected to the server.
pub enum To {
    Origin,
    NonOrigin,
    All,
}

/// Socket: an object used to handle a client's connection with the server.
pub struct Socket<'a> {
    address: SocketAddr,
    clients: &'a ClientMap,
}

impl Socket<'_> {
    /// new: creates a new Socket object
    pub fn new(clients: &ClientMap, address: SocketAddr) -> Socket {
        Socket { address, clients }
    }

    /// send: sends a message to everyone connected to the server matching the
    /// specified "To" scope.
    /// Prints to stderr if a message fails to send. May also return WsError.
    pub fn send(&self, msg: Message, to: To) -> Result<(), WsError> {
        let clients = self.clients.lock()?;
        match to {
            To::Origin => clients
                .get(&self.address)
                .ok_or_else(|| WsError::ClientNotFound)?
                .unbounded_send(msg)?,
            To::NonOrigin => clients
                .iter()
                .filter(|(&client_addr, _)| client_addr != self.address)
                .for_each(|(_, ws_out)| {
                    if let Err(err) = ws_out.unbounded_send(msg.clone()) {
                        eprintln!("send error: {err}");
                    }
                }),
            To::All => clients.iter().for_each(|(_, ws_out)| {
                if let Err(err) = ws_out.unbounded_send(msg.clone()) {
                    eprintln!("send error: {err}");
                }
            }),
        };
        Ok(())
    }
}
