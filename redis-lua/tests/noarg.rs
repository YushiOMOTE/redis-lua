#[tokio::test]
async fn noarg() {
    let script = redis_lua::lua!(
        return 1 + 3 + 10;
    );

    let mut cli = redis::Client::open("redis://127.0.0.1").unwrap();
    let res: usize = script.invoke(&mut cli).unwrap();
    assert_eq!(res, 14);
}
