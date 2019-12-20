use proc_macro_hack::proc_macro_hack;

mod script;

#[proc_macro_hack]
pub use redis_lua_macro::{lua, lua_s};
pub use script::{gen_script, Info, Script, ScriptJoin, TakeScript};
