#[macro_use]
mod util;

#[test]
fn json() {
    test!(String {
        return cjson.encode({["foo"]= "bar"})
    }, "{\"foo\":\"bar\"}");
    test!(bool {
        local p = cjson.decode("{\"foo\":\"bar\"}");
        return p.foo == "bar"
    }, true);
}

#[test]
fn msgpack() {
    test!(Vec<u8> {
        return cmsgpack.pack({"foo", "bar", "baz"})
    }, b"\x93\xa3foo\xa3bar\xa3baz");

    let bytes = b"\x93\xa3foo\xa3bar\xa3baz";
    test!(Vec<String> {
        return cmsgpack.unpack(@bytes);
    }, vec!["foo".to_owned(), "bar".into(), "baz".into()]);
}

#[test]
fn sha1hex() {
    test!(String {
        return redis.sha1hex("foo");
    }, "0beec7b5ea3f0fdbc95d0dd47f3c5bc275da8a33");
}

#[test]
fn log() {
    test!(usize {
        redis.log(redis.LOG_DEBUG, "debug");
        redis.log(redis.LOG_VERBOSE, "verbose");
        redis.log(redis.LOG_NOTICE, "notice");
        redis.log(redis.LOG_WARNING, "warning");
        return 0;
    }, 0usize);
}

// TODO: bitop
