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

### Usage

Add this to your `Cargo.toml`:

```toml
redis-lua = "0.1"
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

Supports the two ways to pass values from Rust to scripts.

* `@x` to capture a Rust variable (by move).
* `$x` to substitute a value later.

```rust
let x = 50;

let script = lua!(
    local v = $a * $b * $c
    if v > @x then
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

#### Script reusability

Script objects returned by `lua!` are clonable if the captured variables are clonable.

### Joining scripts

`+` operator joins two scripts. The scripts are treated as a single script and evaluted atomically in Redis.
The return value of the first script is discarded. Only the return value of the last script is replied by Redis.

```rust
let script1 = redis_lua::lua!(
    return redis.call("set", "a", $x);
);
let script2 = redis_lua::lua!(
    return redis.call("set", "b", $y);
);
let script = script1 + script2;

let mut cli = redis::Client::open("redis://127.0.0.1").unwrap();
let res: String = script.x(20).y(2).invoke(&mut cli).unwrap();
assert_eq!(res, "OK");
```

### Script trait

Any scripts with substitution completed implements `Script` trait. You can pass them around as `Box<dyn Script>`.

```rust
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
```

If you want to join boxed scripts, use `join` methods.

```rust
let joined_boxed = boxed1.join(boxed2).join(boxed3);
```
