//!
//! redis-lua is a Lua scripting helper for [`redis-rs`](https://github.com/mitsuhiko/redis-rs)
//!
//! redis-lua allows to insert Redis Lua scripts in Rust code in safer manner. The key features of redis-lua are:
//!
//! * Compile-time lint for Redis Lua scripts.
//! * Capturing Rust variables in Lua scripts.
//! * Safe argument substitution.
//! * Safely joining multiple Lua scripts.
//!
//! redis-lua requires nightly.
//!
//! # Invoke a Lua script
//!
//! [`lua`][] macro allows to create the Lua script. The script object implements [`Script`][] trait.
//! The return value from the script can be converted to any types which implement [`redis::FromRedisValue`][].
//!
//! ```rust
//! use redis_lua::lua;
//!
//! # fn main() {
//! let mut cli = redis::Client::open("redis://localhost").unwrap();
//!
//! let script = lua!(return 1 + 2);
//! let num: usize = script.invoke(&mut cli).unwrap();
//! assert_eq!(num, 3);
//! # }
//! ```
//!
//! Any Lua syntax supported by Redis Lua is usable.
//!
//! * if-else
//!
//! ```rust
//! # use redis_lua::lua;
//! #
//! # fn main() {
//! # let mut cli = redis::Client::open("redis://localhost").unwrap();
//! #
//! # let script =
//! lua!(
//!   if 3 > 1 then
//!     return 3
//!   else
//!     return 1
//!   end
//! );
//! # let num: usize = script.invoke(&mut cli).unwrap();
//! # assert_eq!(num, 3);
//! # }
//! ```
//!
//! * for-loop
//!
//! ```rust
//! # use redis_lua::lua;
//! #
//! # fn main() {
//! # let mut cli = redis::Client::open("redis://localhost").unwrap();
//! #
//! # let script =
//! lua!(
//!   local sum = 0
//!   for i=1,10 do
//!      sum = sum + i
//!   end
//!   return sum
//! );
//! # let num: usize = script.invoke(&mut cli).unwrap();
//! # assert_eq!(num, 55);
//! # }
//! ```
//!
//! # Error reporting
//!
//! Errors in Lua scripts (such as undefined variables) are detected at compile time.
//!
//! ```rust,ignore
//! # use redis_lua::lua;
//! #
//! # fn main() {
//! # let mut cli = redis::Client::open("redis://localhost").unwrap();
//! #
//! # let script =
//! lua!(
//!   return a + 1
//! );
//! # let num: usize = script.invoke(&mut cli).unwrap();
//! # assert_eq!(num, 55);
//! # }
//! ```
//!
//! ```text,ignore
//! error: in lua: `a` is not defined (undefined_variable)
//!    --> src/lib.rs:80:10
//!    |
//! 10 |   return a + 1
//!    |          ^
//!
//!    error: aborting due to previous error
//! ```
//!
//! # Capturing a variable
//!
//! `@` with an identifier allows to capture a Rust variable in the script. It allows to capture any types which implement [`serde::Serialize`][].
//!
//! ```rust
//! # use redis_lua::lua;
//! #
//! # fn main() {
//! # let mut cli = redis::Client::open("redis://localhost").unwrap();
//! #
//! let x = 10;
//!
//! let script = lua!(return @x + 2);
//! let num: usize = script.invoke(&mut cli).unwrap();
//! assert_eq!(num, 12);
//! # }
//! ```
//!
//! # Argument substitution
//!
//! `$` with an identifier allows to substitute a variable before actually running the script. Same as `@`, any types which implement [`serde::Serialize`][] can be substituted.
//!
//! ```rust
//! # use redis_lua::lua;
//! #
//! # fn main() {
//! # let mut cli = redis::Client::open("redis://localhost").unwrap();
//! #
//! let script = lua!(return $x + 2);
//! let num: usize = script.x(30).invoke(&mut cli).unwrap();
//! assert_eq!(num, 32);
//! # }
//! ```
//!
//! The difference from `@` is that the same script can be called multiple times with different values.
//!
//! ```rust
//! # use redis_lua::lua;
//! #
//! # fn main() {
//! # let mut cli = redis::Client::open("redis://localhost").unwrap();
//! #
//! let script = lua!(return $x + 2);
//!
//! for i in 0..10 {
//!     let num: usize = script.clone().x(i).invoke(&mut cli).unwrap();
//!     assert_eq!(num, i + 2);
//! }
//! # }
//! ```
//!
//! The script object is clonable if all the variables it captures are clonable or it captures no variables.
//!
//! # Type conversion
//!
//! `@` and `$` allow to pass Rust variables to Lua scripts. Primitive types and strings are converted to
//! the corresponding primitive types/strings in Lua scripts.
//! A byte vector such as `&[u8]` is converted to a Lua string.
//!
//! Complicated types such as structs, tuples, maps and non-u8 vectors are converted to Lua tables.
//! The name of struct members become the key of tables.
//!
//! While a single u8 vector is converted to a Lua string, u8 vectors which appear inside a complicated type
//! is converted to Lua tables by default. To convert them to Lua strings, use [`serde_bytes`][] as follows.
//!
//! ```rust
//! # use serde::Serialize;
//! #[derive(Serialize)]
//! struct Data {
//!     table: Vec<u8>,  // This field will be converted to a Lua table of `u8`.
//!     #[serde(with = "serde_bytes")]
//!     string: Vec<u8>, // This field will be converted to a Lua string.
//! }
//! ```
//!

#![feature(specialization)]

use proc_macro_hack::proc_macro_hack;

mod script;
mod types;

pub use futures;
pub use redis;
pub use serde;

/// Macro to embed Lua script in Rust code.
#[proc_macro_hack]
pub use redis_lua_macro::lua;

/// Macro to convert Lua script to string.
#[proc_macro_hack]
pub use redis_lua_macro::lua_s;

pub use script::{gen_script, Info, Script, ScriptJoin, TakeScript};
pub use types::{writer, Writer};
