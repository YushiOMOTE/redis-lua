#[macro_use]
mod util;

#[tokio::test]
async fn vec() {
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
async fn map() {
    let mut map = std::collections::BTreeMap::new();
    map.insert(3, "a");
    map.insert(4, "b");
    map.insert(5, "c");

    test!(Vec<String> {
        return {@map[3], @map[4], @map[5]}
    }, vec!["a".to_owned(), "b".into(), "c".into()]);
}

#[tokio::test]
async fn custom() {
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
