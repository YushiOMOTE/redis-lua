use redis_lua::{lua, lua_str};

fn main() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_connection().unwrap();

    let value = 10;
    let weight = 2;
    let msg = "Calc";

    // Generate a script object
    let script = lua!(
        local res = @weight * @value;
        return @msg..": "..@value.."*"..@weight.."="..res;
    );

    let v: String = script.invoke(&mut con).unwrap();
    println!("return: {}", v);

    // Generate a string
    let v: u32 = redis::Script::new(lua_str!(
        local a = ARGV[1];
        local b = ARGV[2];
        return a + b;
    ))
    .arg(310)
    .arg(42)
    .invoke(&mut con)
    .unwrap();

    println!("return: {}", v);
}
