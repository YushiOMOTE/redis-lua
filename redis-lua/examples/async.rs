use redis_lua::lua;

#[tokio::main]
async fn main() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_multiplexed_tokio_connection().await.unwrap();

    let msg = "Hello Lua";
    let num = 42;

    let script = lua!(
        return @msg .. " / " .. @num
    );

    let v: String = script.invoke_async(&mut con).await.unwrap();
    println!("result: {}", v);
}
