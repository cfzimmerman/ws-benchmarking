pub mod redis_io {
    use redis::{aio::Connection, AsyncCommands, RedisResult};

    use crate::benchmark;

    pub struct RedisConn {
        client: Connection,
    }

    impl RedisConn {
        pub async fn build(addr: &str) -> RedisConn {
            let conn = redis::Client::open(addr).unwrap();
            let client = conn.get_tokio_connection().await.unwrap();
            RedisConn { client }
        }

        pub async fn set_benchmark_payload(&mut self, pl: benchmark::Payload) {
            if let Err(res) = self
                .client
                .set::<&str, benchmark::Counters, i32>(&pl.id, pl.counters)
                .await
            {
                eprintln!("add_benchmark_payload error: {:?}", res);
            };
        }

        pub async fn get_benchmark_payload(&mut self, id: &str) -> RedisResult<benchmark::Payload> {
            let counters: benchmark::Counters =
                self.client.get::<&str, benchmark::Counters>(id).await?;
            Ok(benchmark::Payload {
                id: benchmark::Payload::redis_key(id).to_string(),
                counters,
            })
        }
    }
}

pub mod benchmark {
    use std::sync::Arc;

    use my_ws::ws::{
        event::EventAction,
        socket::{Socket, To},
        ws_io::Message,
    };
    use redis_macros::{FromRedisValue, ToRedisArgs};
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    use crate::redis_io::RedisConn;

    #[derive(Serialize, Deserialize, FromRedisValue, ToRedisArgs, Debug)]
    pub struct Counters {
        pub current: i32,
        pub to: i32,
    }

    #[derive(Serialize, Deserialize, FromRedisValue, ToRedisArgs, Debug)]
    pub struct Payload {
        pub id: String,
        pub counters: Counters,
    }

    impl Payload {
        pub fn redis_key(id: &str) -> &str {
            // This looks ridiculous, but it allows us to change the ID format later if we want.
            id
        }
    }

    /// handle_message: Our benchmark message handler. Receives a message, parses it,
    /// sends it to Redis, pulls it from Redis, stringifies it, and returns it to the
    /// client.
    pub fn handle_message(redis: Arc<tokio::sync::Mutex<RedisConn>>) -> EventAction {
        Box::new(
            move |socket: Arc<tokio::sync::Mutex<Socket>>, message: serde_json::Value| {
                let redis = redis.clone();
                let payload = if let Ok(pl) = serde_json::from_value::<Payload>(message) {
                    pl
                } else {
                    // eprintln!("unable to parse: {}", message);
                    return;
                };
                tokio::spawn(async move {
                    let mut rd = redis.lock().await;
                    let socket = socket.lock().await;

                    let id = payload.id.clone();
                    rd.set_benchmark_payload(payload).await;
                    let res = rd.get_benchmark_payload(&id).await;
                    match serde_json::to_string(&res.unwrap()) {
                        Ok(res_str) => {
                            if let Err(err) = socket.send(Message::Text(res_str), To::Origin).await
                            {
                                eprintln!("error sending message: {:?}", err);
                            };
                        }
                        Err(err) => {
                            eprintln!("parse error: {:?}", err);
                        }
                    };
                });
            },
        )
    }
}
