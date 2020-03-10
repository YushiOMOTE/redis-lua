#[tokio::test]
async fn boxed_add() {
    let x = 10;
    let y = 20;
    let z = 30;

    use redis_lua::Script;

    let script1 = Box::new(redis_lua::lua!(
        return @x + 10;
    )) as Box<dyn redis_lua::Script>;
    let script2 = Box::new(redis_lua::lua!(
        return @y + 10;
    )) as Box<dyn redis_lua::Script>;
    let script3 = Box::new(redis_lua::lua!(
        return @z + 10;
    )) as Box<dyn redis_lua::Script>;
    let script = script1.join(script2).join(script3);

    let mut cli = redis::Client::open("redis://127.0.0.1").unwrap();
    let res: usize = script.invoke(&mut cli).unwrap();
    assert_eq!(res, 40);
}
