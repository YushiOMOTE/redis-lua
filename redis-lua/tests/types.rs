#[macro_use]
mod util;

#[tokio::test]
async fn arg_boolean() {
    // According to https://redis.io/commands/eval,
    // Lua boolean true becomes Redis integer reply with value of 1.

    let p = false;
    test!(usize {
        return @p
    }, 0);

    let p = true;
    test!(usize {
        return @p
    }, 1);
}

#[tokio::test]
async fn arg_number() {
    macro_rules! arg_number {
        ($v:expr; $($t:ty),*) => {
            $(
                let p = $v as $t;
                test!($t {
                    return @p
                }, $v);
            )*
        };
    }

    arg_number!(0; i8, i16, i32, i64, u8, u16, u32, u64);
    arg_number!(11; i8, i16, i32, i64, u8, u16, u32, u64);
    arg_number!(-23; i8, i16, i32, i64);
    arg_number!(0.0; f32, f64);
    arg_number!(-10.13; f32, f64);
    arg_number!(84.141; f32, f64);
}

#[tokio::test]
async fn arg_str() {
    let p = "hello".to_string();
    test!(String {
        return @p
    }, "hello".to_string());

    let p = b"hello";
    test!(Vec<u8> {
        return @p
    }, b"hello");
}

#[tokio::test]
async fn arg_opt() {
    let p = Some("hello".to_string());
    test!(Option<String> {
        return @p
    }, Some("hello".to_string()));

    let p = None::<u32>;
    test!(usize {
        if @p == "" then return 1 else return 0 end
    }, 1);

    let p = Some(b"hello");
    test!(Option<Vec<u8>> {
        return @p
    }, Some(b"hello".to_vec()));
}

#[tokio::test]
async fn arg_vec() {
    let vec = vec![1usize, 2, 3];
    test!(usize {
        local sum = 0
        for i = 1,3 do
            sum = sum + @vec[i]
        end
        return sum
    }, 6);

    let vec = vec![2usize, 4, 6];
    test!((usize, usize) {
        local sum1 = 0
        local sum2 = 0
        for k, v in ipairs(@vec) do
            sum1 = sum1 + v
            sum2 = sum2 + k
        end
        return {sum1, sum2}
    }, (12, 6));

    let vec = Vec::<usize>::new();
    test!(usize {
        local c = 0
        for _ in ipairs(@vec) do
            c = c + 1
        end
        return c
    }, 0);
}

#[tokio::test]
async fn arg_map() {
    let mut map = std::collections::BTreeMap::new();
    map.insert(3, "a");
    map.insert(4, "b");
    map.insert(5, "c");

    test!(Vec<String> {
        return {@map[3], @map[4], @map[5]}
    }, vec!["a".to_owned(), "b".into(), "c".into()]);
}

#[tokio::test]
async fn arg_custom() {
    #[derive(serde::Serialize)]
    struct A {
        a: usize,
        b: String,
        c: bool,
    }

    let a = A {
        a: 32,
        b: "hello".into(),
        c: true,
    };

    test!((bool, String, usize) {
        local a = @a;
        return {a["c"], a["b"], a["a"]}
    }, (true, "hello".into(), 32));
}

#[tokio::test]
async fn arg_nested_vec() {
    // vec in vec
    let a = rmp_serde::to_vec(&1).unwrap();
    let b = rmp_serde::to_vec(&2).unwrap();
    let c = rmp_serde::to_vec(&3).unwrap();
    let vec = vec![a, b, c];
    test!(usize {
        return cmsgpack.unpack(@vec[1]) + cmsgpack.unpack(@vec[2]) + cmsgpack.unpack(@vec[3])
    }, 6);

    // vec in map
    let mut map = std::collections::BTreeMap::new();
    map.insert(3, rmp_serde::to_vec(&1).unwrap());
    map.insert(4, rmp_serde::to_vec(&2).unwrap());
    map.insert(5, rmp_serde::to_vec(&3).unwrap());
    test!(Vec<usize> {
        return {cmsgpack.unpack(@map[3]), cmsgpack.unpack(@map[4]), cmsgpack.unpack(@map[5])}
    }, vec![1, 2, 3]);

    // vec in struct
    #[derive(serde::Serialize)]
    struct A {
        a: usize,
        b: String,
        c: Vec<u8>,
    }
    let a = A {
        a: 32,
        b: "hello".into(),
        c: rmp_serde::to_vec("OK").unwrap(),
    };
    test!((String, String, usize) {
        local a = @a;
        return {cmsgpack.unpack(a["c"]), a["b"], a["a"]}
    }, ("OK".into(), "hello".into(), 32));
}
