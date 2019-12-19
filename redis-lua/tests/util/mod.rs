#![allow(unused)]

use redis::ScriptInvocation;

pub fn run<T: redis::FromRedisValue, S: redis_lua::Script>(script: S) -> T {
    let cli = redis::Client::open("redis://127.0.0.1").unwrap();
    let mut con = cli.get_connection().unwrap();
    script.invoke(&mut con).unwrap()
}

macro_rules! test {
    ($type:ty { $($t:tt)* }, $exp:expr) => {{
        assert_eq!(crate::util::run::<$type, _>(redis_lua::lua! {
            $($t)*
        }), $exp)
    }}
}
