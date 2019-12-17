use redis::ScriptInvocation;

pub fn run<T: redis::FromRedisValue, F>(script: redis_lua::Script<F>) -> T
where
    F: FnOnce(ScriptInvocation) -> ScriptInvocation,
{
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
