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

        pub async fn set_benchmark_payload(&mut self, pl: benchmark::Payload) -> () {
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
    use redis_macros::{FromRedisValue, ToRedisArgs};
    use serde::{Deserialize, Serialize};

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

        pub fn redis_values(&self) -> [(&str, i32); 2] {
            [("current", self.counters.current), ("to", self.counters.to)]
        }
    }
}
