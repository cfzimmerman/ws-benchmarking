use crate::ws::socket::Socket;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// Event: an object specifying to listen for a certain type of
/// event with instructions for how to handle the event.
pub struct Event {
    pub path: &'static str,
    pub action: EventAction,
}

/// EventMap: A thread-shared hash mapping each path to its associated action.
pub type EventMap = Arc<Mutex<HashMap<&'static str, EventAction>>>;

/// EventAction: Given a Socket instance and the string received by the server,
/// performs some desired action.
pub type EventAction = Box<dyn Fn(&Socket, String) -> () + Send>;

impl Event {
    /// new: Creates a new Event object.
    pub fn new(path: &'static str, action: EventAction) -> Event {
        Event { path, action }
    }
}
