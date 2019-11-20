use redis_lua::{lua, lua_f, lua_s};

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
    let v: u32 = redis::Script::new(lua_s!(
        local a = ARGV[1];
        local b = ARGV[2];
        return a + b;
    ))
    .arg(310)
    .arg(42)
    .invoke(&mut con)
    .unwrap();

    println!("return: {}", v);

    // Generate a script object
    let script1 = lua_f!(
        local v = @a * @b
        if v > 80 then
            return v .. " > 80"
        else
            return v .. " <= 80"
        end
    );

    let script2 = lua_f!(
        local v = @a * @b * @c
        if v > 50 then
            return v .. " > 50"
        else
            return v .. " <= 50"
        end
    );

    for i in 0..4 {
        let r: String = script1.a(15).b(i).invoke(&mut con).unwrap();
        println!("{}", r);
    }

    for i in 0..4 {
        let r: String = script2.a(15).b(i).c(3).invoke(&mut con).unwrap();
        println!("{}", r);
    }
}
