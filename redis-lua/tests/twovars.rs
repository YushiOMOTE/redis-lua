#[tokio::test]
async fn twovars() {
    let script = redis_lua::lua!(
        return $a + $b + 10;
    );

    let mut cli = redis::Client::open("redis://127.0.0.1").unwrap();
    let res: usize = script.a(20).b(11).invoke(&mut cli).unwrap();
    assert_eq!(res, 41);
}
