#[tokio::test]
async fn onearg() {
    let a = 10;
    let script = redis_lua::lua!(
        return @a + 10;
    );

    let mut cli = redis::Client::open("redis://127.0.0.1").unwrap();
    let res: usize = script.invoke(&mut cli).unwrap();
    assert_eq!(res, 20);
}
