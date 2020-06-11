use redis_lua::{lua, Script};

async fn do1(script: Box<dyn Script>) {
    let mut cli = redis::Client::open("redis://127.0.0.1").unwrap();
    let res: usize = script.invoke(&mut cli).unwrap();
    assert_eq!(res, 3);
}

async fn do2(script: Box<dyn Script>) {
    let mut cli = redis::Client::open("redis://127.0.0.1").unwrap();
    let res: usize = script.invoke(&mut cli).unwrap();
    assert_eq!(res, 12);
}

async fn do3(script: Box<dyn Script>) {
    let mut cli = redis::Client::open("redis://127.0.0.1").unwrap();
    let res: usize = script.invoke(&mut cli).unwrap();
    assert_eq!(res, 4);
}

#[tokio::test]
async fn pass_around() {
    let script1 = lua! {
        return 1 + 2;
    };

    let x = 10;
    let script2 = lua! {
        return @x + 2;
    };

    let incomplete_script = lua! {
        return $x + 2;
    };
    let script3 = incomplete_script.x(2);

    let boxed1 = Box::new(script1) as Box<dyn redis_lua::Script>;
    let boxed2 = Box::new(script2) as Box<dyn redis_lua::Script>;
    let boxed3 = Box::new(script3) as Box<dyn redis_lua::Script>;

    do1(boxed1).await;
    do2(boxed2).await;
    do3(boxed3).await;
}
