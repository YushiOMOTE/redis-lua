#[test]
fn addvar() {
    let script1 = redis_lua::lua!(
        return $x + 10;
    );
    let script2 = redis_lua::lua!(
        return $y + 10;
    );
    let script = script1 + script2;

    let mut cli = redis::Client::open("redis://127.0.0.1").unwrap();
    let res: usize = script.x(20).y(2).invoke(&mut cli).unwrap();
    assert_eq!(res, 12);
}
