use redis::AsyncCommands;

#[tokio::test]
async fn array() {
    let cli = redis::Client::open("redis://127.0.0.1").unwrap();
    let mut cli = cli.get_multiplexed_tokio_connection().await.unwrap();

    let (ks, vs): (Vec<_>, Vec<_>) = (0..10)
        .map(|i| {
            let k = format!("key:{}", i);
            let d = rmp_serde::Raw::from_utf8(rmp_serde::to_vec(&format!("data:{}", i)).unwrap());
            (k, d)
        })
        .unzip();

    let script = redis_lua::lua!(
        for i, k in ipairs(@ks) do
            redis.call("set", k, @vs[i])
            end
    );
    let _: () = script.invoke_async(&mut cli).await.unwrap();

    for i in 0..10 {
        let v: Vec<u8> = cli.get(format!("key:{}", i)).await.unwrap();
        let e = rmp_serde::to_vec(&format!("data:{}", i)).unwrap();
        assert_eq!(v, e);
    }
}
