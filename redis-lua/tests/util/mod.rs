#![allow(unused)]

use redis::ScriptInvocation;

pub async fn run<T: redis::FromRedisValue + Send, S: redis_lua::Script + Send>(script: S) -> T {
    let cli = redis::Client::open("redis://127.0.0.1").unwrap();
    let mut con = cli.get_multiplexed_tokio_connection().await.unwrap();
    script.invoke_async(&mut con).await.unwrap()
}

macro_rules! test {
    ($type:ty { $($t:tt)* }, $exp:expr) => {{
        assert_eq!(crate::util::run::<$type, _>(redis_lua::lua! {
            $($t)*
        }).await, $exp);
    }}
}
