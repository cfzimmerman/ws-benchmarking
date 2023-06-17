/*

Build: manages the TCP connection
Listen: accepts new clients and adds them to the hashmap
    On an event, searches for the appropriate handler. Passes the event to them.
Events: Handlers for pre-defined routes.
        Provide a client handler for the event

*/

pub mod ws;
