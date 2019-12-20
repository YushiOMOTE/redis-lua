#[test]
fn addvar() {
    let script1 = redis_lua::lua!(
        return redis.call("set", "a", $x);
    );
    let script2 = redis_lua::lua!(
        return redis.call("set", "b", $y);
    );
    let script = script1 + script2;

    let mut cli = redis::Client::open("redis://127.0.0.1").unwrap();
    let res: String = script.x(20).y(2).invoke(&mut cli).unwrap();
    assert_eq!(res, "OK");
}
