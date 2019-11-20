#![feature(proc_macro_diagnostic)]

extern crate proc_macro;

use self::proc_macro::TokenStream;
use quote::quote;

mod check;
mod file;
mod script;
mod token;

use crate::{check::Checker, script::Script};

use proc_macro_hack::proc_macro_hack;

#[proc_macro_hack]
pub fn lua_str(input: TokenStream) -> TokenStream {
    let script = Script::new(input, false);

    Checker::new().define("ARGV").check(&script);

    let script = script.script();
    let script_code = quote! {
        #script
    };
    script_code.into()
}

#[proc_macro_hack]
pub fn lua(input: TokenStream) -> TokenStream {
    let script = Script::new(input, true);

    Checker::new()
        .defines(
            script
                .args()
                .iter()
                .map(|arg| arg.as_lua().into())
                .collect(),
        )
        .check(&script);

    let args: Vec<_> = script
        .args()
        .iter()
        .map(|arg| {
            let arg = arg.as_rust();
            // TODO: Remove `clone` requirement
            quote! {
                invoke.arg(#arg);
            }
        })
        .collect();

    let script = script.wrap();

    let script_code = quote! {
        redis_lua::Script::new(#script, move |mut invoke| {
            #(#args)*
            invoke
        })
    };
    script_code.into()
}
