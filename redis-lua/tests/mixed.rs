#[tokio::test]
async fn mixed() {
    let y = 3;
    let script = redis_lua::lua!(
        return $x + @y + 10;
    );

    let mut cli = redis::Client::open("redis://127.0.0.1").unwrap();
    let res: usize = script.x(4).invoke(&mut cli).unwrap();
    assert_eq!(res, 17);
}
