use redis::ScriptInvocation;
use redis_lua::lua;

fn run<T: redis::FromRedisValue, F>(script: redis_lua::Script<F>) -> T
where
    F: FnOnce(ScriptInvocation) -> ScriptInvocation,
{
    let cli = redis::Client::open("redis://127.0.0.1").unwrap();
    let mut con = cli.get_connection().unwrap();
    script.invoke(&mut con).unwrap()
}

#[test]
fn assign1() {
    let s = lua! {
        local a = 1;
        return a;
    };
    assert_eq!(run::<usize, _>(s), 1usize);
}

#[test]
fn assign2() {
    let s = lua! {
        local a = 1;
        local b = 2;
        return a + b;
    };
    assert_eq!(run::<usize, _>(s), 3usize);
}

#[test]
fn assign3() {
    let s = lua! {
        local a = 1;
        local b = a*2;
        return b;
    };
    assert_eq!(run::<usize, _>(s), 2usize);
}

#[test]
fn assign4() {
    let s = lua! {
        local a = 4; local b = a*2;
        return b;
    };
    assert_eq!(run::<usize, _>(s), 8usize);
}

#[test]
fn assign5() {
    let s = lua! {
        local a = 3 local b = a*2;
        return b;
    };
    assert_eq!(run::<usize, _>(s), 6usize);
}

#[test]
fn func1() {
    let s = lua! {
        redis.log(redis.LOG_DEBUG, "debug");
        redis.log(redis.LOG_VERBOSE, "verbose");
        redis.log(redis.LOG_NOTICE, "notice");
        redis.log(redis.LOG_WARNING, "warning");
        return 0;
    };
    assert_eq!(run::<usize, _>(s), 0usize);
}

#[test]
fn func2() {
    let s = lua! {
        local logtable = {}

        local function logit(msg)
            logtable[#logtable+1] = msg
        end

        logit("foo")
        logit("bar")

        return logtable
    };
    assert_eq!(
        run::<Vec<String>, _>(s),
        vec!["foo".to_owned(), "bar".into()]
    );
}

#[test]
fn bool1() {
    assert_eq!(run::<bool, _>(lua! { return true }), true);
    assert_eq!(run::<bool, _>(lua! { return false }), false);
}

#[test]
fn bool2() {
    assert_eq!(run::<bool, _>(lua! { return 1 }), true);
    assert_eq!(run::<bool, _>(lua! { return 0 }), false);
    assert_eq!(run::<bool, _>(lua! { return nil }), false);
}

#[test]
fn num1() {
    assert_eq!(run::<usize, _>(lua! { return 4 }), 4);
    // Indeed, Redis protocol doesn't support floating points.
    assert_eq!(run::<f64, _>(lua! { return 0.4 }), 0.0);
    assert_eq!(run::<bool, _>(lua! { return 0.4 == 0.4 }), true);
    assert_eq!(run::<bool, _>(lua! { return 0.457e-3 == 0.457e-3 }), true);
    assert_eq!(run::<bool, _>(lua! { return 0.3e12 == 0.3e12 }), true);
    // FIXME: Seems not to be supported by full-moon at the moment.
    // assert_eq!(run::<bool, _>(lua! { return 5e+20 == 5e+20 }), true);
}

#[test]
fn string1() {
    assert_eq!(run::<String, _>(lua! { return "one string" }), "one string");
    // Indeed, a single quotation is not supported by Rust.
    // assert_eq!(run::<String, _>(lua! { return 'one string' }), "one string");
    assert_eq!(
        run::<String, _>(lua! {
            local s1 = "one string";
            local s2 = string.gsub(s1, "one", "another");
            return s2
        }),
        "another string"
    );
}

#[test]
fn string2() {
    assert_eq!(
        run::<String, _>(lua! { return "one line\nnext\"in quotes\", 'in quotes'" }),
        "one line\nnext\"in quotes\", 'in quotes'"
    );

    // FIXME: This is not supported.
    // let s = run::<String, _>(lua! {
    //     page = [[
    // <HTML>
    // <HEAD>
    // <TITLE>An HTML Page</TITLE>
    // </HEAD>
    // <BODY>
    //  <A HREF="http://www.lua.org">Lua</A>
    //  [[a text between double brackets]]
    // </BODY>
    // </HTML>
    //     ]]
    //         return page
    // });
    // println!("{}", s);
}

#[test]
fn string3() {
    assert_eq!(run::<usize, _>(lua! { return "10" + 1 }), 11);
    assert_eq!(run::<String, _>(lua! { return "10 + 1" }), "10 + 1");
    assert_eq!(
        run::<bool, _>(lua! { return "-5.3e-10"*"2" == -1.06e-09 }),
        true
    );
    assert_eq!(run::<String, _>(lua! { return 10 .. 20 }), "1020");
    assert_eq!(run::<String, _>(lua! { return tostring(10) }), "10");
}

#[test]
fn table1() {
    assert_eq!(
        run::<Vec<String>, _>(lua! { return {} }),
        Vec::<String>::new()
    );
    assert_eq!(
        run::<usize, _>(lua! { local a = {} local k = "x" a[k] = 10 return a[k] }),
        10
    );
    assert_eq!(
        run::<usize, _>(lua! { local a = {} local k = "x" a[k] = 10 return a["x"] + a["x"] }),
        20
    );
    assert_eq!(
        run::<usize, _>(lua! { local a = {} a["x"] = 10 local b = a a["x"] = 20 return b["x"] }),
        20
    );
    assert_eq!(
        run::<Vec<String>, _>(lua! { local a = {} a["x"] = 10 local b = a a = nil return b }),
        Vec::<String>::new(),
    );

    let v: Vec<_> = (1usize..=1000).map(|i| i * 2).collect();
    assert_eq!(
        run::<Vec<usize>, _>(lua! {
            local a = {}
            for i=1,1000 do a[i] = i * 2 end
            return a
        }),
        v,
    );
}
