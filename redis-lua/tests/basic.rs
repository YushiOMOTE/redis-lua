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

macro_rules! test {
    ($type:ty { $($t:tt)* }, $exp:expr) => {{
        assert_eq!(run::<$type, _>(lua! {
            $($t)*
        }), $exp)
    }}
}

#[test]
fn ops_assign() {
    // assign number
    test!(i32 {
        local a = 3;
        return  a;
    }, 3);
    test!(i32 {
        local a = -4;
        return  a;
    }, -4);

    // assign combination
    test!(usize {
        local a = 1;
        local b = 2;
        return a + b;
    }, 3);
    test!(usize {
        local a = 1;
        local b = a*2;
        return b;
    }, 2);
    test!(usize {
        local a = 4; local b = a*2;
        return b;
    }, 8);
    test!(usize {
        local a = 3 local b = a*2;
        return b;
    }, 6);

    // assign string
    test!(String {
        local a = "ok";
        return  a;
    }, "ok");
    test!(String {
        local a = "abc"; local b = "def";
        return a .. b;
    }, "abcdef");

    // assign function
    test!(String {
        local function foo()
            return "OOOKKK"
        end
        local ok = foo();
        return ok
    }, "OOOKKK");

    // multiple-assign
    test!(usize {
        local x = 33
        local a, b = 10, 2*x
        return a + b
    }, 76);
    test!(Option<usize> {
        local a, b, c = 1, 2
        return c
    }, None);
    test!(Vec<usize> {
        local a, b = 1, 2, 3;
        return {a, b}
    }, vec![1, 2]);
    test!(Vec<usize> {
        local function foo()
            return 1, 2
        end
        local x, y = foo()
        return {x, y}
    }, vec![1, 2]);

    // swap
    test!(Vec<usize> {
        local x = 3;
        local y = 9;
        x, y = y, x;
        return {x, y}
    }, vec![9, 3]);
    test!(Vec<usize> {
        local p = {3, 9};
        local i = 1;
        local j = 2;
        p[i], p[j] = p[j], p[i];
        return p
    }, vec![9, 3]);
}

#[test]
fn ops_arith_add() {
    test!(i32 { return  3 + 4 }, 7);
    test!(i32 { return  -3 + 4 }, 1);
    test!(i32 { return  3 + -4 }, -1);
    test!(i32 { return  -3 + -4 }, -7);
}

#[test]
fn ops_arith_sub() {
    test!(i32 { return  3 - 4; }, -1);
    test!(i32 { return  -3 - 4; }, -7);
    test!(i32 { return  3 - -4; }, 7);
    test!(i32 { return  -3 - -4; }, 1);
}

#[test]
fn ops_arith_mul() {
    test!(i32 { return  3 * 4; }, 12);
    test!(i32 { return  -3 * 4; }, -12);
    test!(i32 { return  3 * -4; }, -12);
    test!(i32 { return  -3 * -4; }, 12);
}

#[test]
fn ops_arith_div() {
    test!(i32 { return  4 / 4; }, 1);
    test!(i32 { return  -8 / 4; }, -2);
    test!(i32 { return  8 / -4; }, -2);
    test!(i32 { return  -8 / -4; }, 2);
}

#[test]
fn ops_rel() {
    test!(bool { return  4 == 4; }, true);
    test!(bool { return  4 ~= 4; }, false);
    test!(bool { return  4 > 4; }, false);
    test!(bool { return  -8 < 4; }, true);
    test!(bool { return  8 >= -4; }, true);
    test!(bool { return  -8 <= -4; }, true);
}

#[test]
fn ops_logical() {
    test!(u32 { return 4 and 5; }, 5);
    test!(Option<u32> { return nil and 5; }, None);
    test!(usize { return 4 or 5; }, 4);
    test!(usize { return false or 5; }, 5);
    test!(bool { return not false; }, true);
}

#[test]
fn ops_trinary() {
    test!(u32 { return true and 3 or 2; }, 3);
    test!(u32 {
        local x = 10;
        local y = 11;
        return (x > y) and 1 or 2;
    }, 2);
}

#[test]
fn ops_concat() {
    test!(String { return "hello" .. "world"; }, "helloworld");
    test!(String { local a = "hello" return a .. "world"; }, "helloworld");
}

#[test]
fn ops_precedence() {
    test!(bool {
        local a = 8;
        local b = 10;
        local i = 4;
        local p = a+i < b/2+1;
        local q = (a+i) < ((b/2)+1);
        return p == q
    }, true);

    test!(bool {
        local x = 111;
        local p = 5+x^2*8;
        local q = 5+((x^2)*8);
        return p == q
    }, true);

    test!(bool {
        local a = 10;
        local y = 3;
        local z = 5;
        local p = a < y and y <= z;
        local q = (a < y) and (y <= z);
        return p == q
    }, true);

    test!(i32 { local x = 2; return -x^2; }, -4);
    test!(i32 { local x = 2; return (-x)^2; }, 4);

    test!(u64 {
        local x = 2;
        local y = 3;
        local z = 2;
        return x^y^z;
    }, 512);

    test!(u64 {
        local x = 2;
        local y = 3;
        local z = 2;
        return (x^y)^z;
    }, 64);
}

#[test]
fn table_ctor() {
    test!(Vec<String> {
        return {"A", "B", "C"}
    }, vec!["A".to_owned(), "B".into(), "C".into()]);
    test!(Vec<String> {
        return {[1]="A", [2]="B", [3]="C"}
    }, vec!["A".to_owned(), "B".into(), "C".into()]);
    test!(Vec<usize> {
        return {math.abs(-1), math.abs(-2), math.abs(-3)}
    }, vec![1usize, 2, 3]);

    // Redis discards the index 0.
    // 127.0.0.1:6379> eval 'return {[0]="A", "B", "C"}' 0
    // 1) "B"
    // 2) "C"
    test!(Vec<String> {
        return {[0]="A", "B", "C"}
    }, vec!["B".to_owned(), "C".into()]);

    // Having only non-number keys result in an empty list or set in redis.
    // 127.0.0.1:6379> eval "return {x=10, y=10}" 0
    // (empty list or set)
    test!(Vec<String> { return {x=10, y=100} }, Vec::<String>::new());
    test!(Vec<String> { return {["x"]=10, ["y"]=100} }, Vec::<String>::new());

    test!(bool {
        local a = {x=10, y=100}
        return a.x == 10 and a.y == 100
    }, true);
    test!(bool {
        local a = {["x"]=1000, ["y"]=100}
        return a.x == 1000 and a.y == 100
    }, true);
    test!(bool {
        local p = {color="blue", thickness=2, npoints=4,
                   {x=0,   y=0},
                   {x=-10, y=0},
                   {x=-10, y=1},
                   {x=0,   y=1}};
        return p.color == "blue" and p.thickness == 2 and p.npoints == 4
            and p[1].x == 0 and p[1].y == 0
            and p[2].x == -10 and p[2].y == 0
            and p[3].x == -10 and p[3].y == 1
            and p[4].x == 0 and p[4].y == 1
    }, true);

    test!(bool {
        local opnames = {["+"] = "add", ["-"] = "sub",
                   ["*"] = "mul", ["/"] = "div"}
        local i = 20; local s = "-";
        local a = {[i+0] = s, [i+1] = s..s, [i+2] = s..s..s}
        return opnames["-"] == "sub" and a[22] == "---"
    }, true);

    test!(Vec<String> { return {x=10, y=45; "one", "two", "three"} },
          vec!["one".to_owned(), "two".into(), "three".into()]);
}

// #[test]
// fn ctrl_while() {
//     test!(bool {
//         local r = "prefix";
//         local l = {"a", "b", "c"};
//         while l do
//             r = r .. l.value
//     }, true);
// }

#[test]
fn func1() {
    test!(usize {
        redis.log(redis.LOG_DEBUG, "debug");
        redis.log(redis.LOG_VERBOSE, "verbose");
        redis.log(redis.LOG_NOTICE, "notice");
        redis.log(redis.LOG_WARNING, "warning");
        return 0;
    }, 0usize);
}

#[test]
fn func2() {
    test!(Vec<String> {
        local logtable = {}

        local function logit(msg)
            logtable[#logtable+1] = msg
        end

        logit("foo")
        logit("bar")

        return logtable
    }, vec!["foo".to_owned(), "bar".into()]);
}

#[test]
fn bool1() {
    test!(bool { return true }, true);
    test!(bool { return false }, false);
}

#[test]
fn bool2() {
    test!(bool { return 1 }, true);
    test!(bool { return 0 }, false);
    test!(bool { return nil }, false);
}

#[test]
fn num1() {
    test!(usize { return 4 }, 4);
    // Indeed, Redis protocol doesn't support floating points.
    test!(f64 { return 0.4 }, 0.0);
    test!(bool { return 0.4 == 0.4 }, true);
    test!(bool { return 0.457e-3 == 0.457e-3 }, true);
    test!(bool { return 0.3e12 == 0.3e12 }, true);
    // FIXME: Seems not to be supported by full-moon at the moment.
    // test!(bool { return 5e+20 == 5e+20 }, true);
}

#[test]
fn string1() {
    test!(String { return "one string" }, "one string");
    // Indeed, a single quotation is not supported by Rust.
    // test!(String { return 'one string' }, "one string");
    test!(
        String {
            local s1 = "one string";
            local s2 = string.gsub(s1, "one", "another");
            return s2
        },
        "another string"
    );
}

#[test]
fn string2() {
    test!(String { return "one line\nnext\"in quotes\", 'in quotes'" },
          "one line\nnext\"in quotes\", 'in quotes'");

    // FIXME: This is not supported by full-moon.
    // let s = String {
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
    test!(usize { return "10" + 1 }, 11);
    test!(String { return "10 + 1" }, "10 + 1");
    test!(bool { return "-5.3e-10"*"2" == -1.06e-09 }, true);
    test!(String { return 10 .. 20 }, "1020");
    test!(String { return tostring(10) }, "10");
}

#[test]
fn table() {
    test!(Vec<String> { return {} }, Vec::<String>::new());
    test!(usize { local a = {} local k = "x" a[k] = 10 return a[k] }, 10);
    test!(usize { local a = {} local k = "x" a[k] = 10 return a["x"] + a["x"] }, 20);
    test!(usize { local a = {} a["x"] = 10 local b = a a["x"] = 20 return b["x"] }, 20);
    test!(Vec<String> { local a = {} a["x"] = 10 local b = a a = nil return b },
          Vec::<String>::new());
    let v: Vec<_> = (1usize..=1000).map(|i| i * 2).collect();
    test!(Vec<usize> {
        local a = {}
        for i=1,1000 do a[i] = i * 2 end
        return a
    }, v);
}
