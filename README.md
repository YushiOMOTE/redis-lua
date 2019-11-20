# redis-lua

Lua scripting helper for redis-rs.

* Compile-time lint for redis lua script.
* Capturing rust variables in lua script.

```rust
use redis_lua::lua;

fn main() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_connection().unwrap();

    let msg = "Hello Lua";
    let num = 42;

    let script = lua!(
        return @msg .. " / " .. @num
    );

    let v: String = script.invoke(&mut con).unwrap();
    println!("result: {}", v);
}
```

### Reporting errors

Errors in the lua script (such as undefined variables) are detected at compile time.

```rust
    let script = lua!(
        return @msg .. " / " .. @num .. x
    );
```

```
$ cargo build
...
error: in lua: `x` is not defined (undefined_variable)     
  --> redis-lua/examples/simple.rs:11:41                   
   |                         
11 |         return @msg .. " / " .. @num .. x             
   |                                         ^             

error: aborting due to previous error                      
```

### Reusing scripts

```rust
let script = lua_f!(
    local v = @a * @b * @c
    if v > 50 then
        return v .. " > 50"
    else
        return v .. " <= 50"
    end
);

for i in 0..4 {
    // You can call script multiple times with different parameters.
    let r: String = script.a(15).b(i).c(3).invoke(&mut con).unwrap();
    println!("{}", r);
}
```
