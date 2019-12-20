# redis-lua

Lua scripting helper for [redis-rs](https://github.com/mitsuhiko/redis-rs).

* Compile-time lint for Redis Lua script.
* Capturing Rust variables in Lua script.
* Safe argument substitution.

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

Errors in the Lua script (such as undefined variables) are detected at compile time.

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

### Argument substitution

* `@x` to capture a Rust variable (by move).
* `$x` to substitute a value later.

```rust
let x = 50;

let script = lua!(
    local v = $a * $b * $c
    if v > $@ then
        return v .. " is large"
    else
        return v .. " is small"
    end
);

for i in 0..4 {
    // You can `clone()` the script to call it multiple times with different parameters.
    let r: String = script.clone().a(15).b(i).c(3).invoke(&mut con).unwrap();
    println!("{}", r);
}
```
