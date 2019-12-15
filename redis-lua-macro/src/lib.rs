#![feature(proc_macro_diagnostic)]

extern crate proc_macro;

use self::proc_macro::{Ident, TokenStream, TokenTree};
use itertools::Itertools;
use quote::quote;

mod check;
mod file;
mod script;
mod token;

use crate::{
    check::Checker,
    script::{Arg, Script},
};

use proc_macro_hack::proc_macro_hack;

fn to_ident(tt: &TokenTree) -> proc_macro2::TokenStream {
    let s: TokenStream = tt.clone().into();
    s.into()
}

fn to_new_type(tt: &TokenTree) -> proc_macro2::TokenStream {
    let s = format!("___new_type_{}", tt);
    let s: TokenStream = TokenTree::Ident(Ident::new(&s, tt.span())).into();
    s.into()
}

#[proc_macro_hack]
pub fn lua_s(input: TokenStream) -> TokenStream {
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
            let arg = to_ident(arg.as_rust());
            quote! {
                invoke.arg(#arg);
            }
        })
        .collect();

    let script = script.wrap();

    let script_code = quote! {
        ::redis_lua::Script::new(#script, move |mut invoke| {
            #(#args)*
            invoke
        })
    };
    script_code.into()
}

#[proc_macro_hack]
pub fn lua_f(input: TokenStream) -> TokenStream {
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

    // Generate chain method return value & type.
    let rets = |arg: Option<&Arg>, inner| match arg {
        Some(arg) => {
            let new_type = to_new_type(arg.as_rust());
            (
                quote! {
                    #new_type<'a>
                },
                quote! {
                    #new_type(#inner)
                },
            )
        }
        None => (
            quote! {
                ::redis::ScriptInvocation<'a>
            },
            quote! {
                #inner
            },
        ),
    };

    // Generate structs for method chain.
    let chain: Vec<_> = script
        .args()
        .iter()
        .peekable()
        .batching(|iter| {
            let arg = iter.next()?;
            let new_type = to_new_type(arg.as_rust());
            let method = to_ident(arg.as_rust());

            let (ret_type, ret_val) = rets(
                iter.peek().map(|r| *r),
                quote! {
                    self.0
                },
            );

            Some(quote! {
                struct #new_type<'a>(::redis::ScriptInvocation<'a>);

                impl<'a> #new_type<'a> {
                    fn #method(mut self, v: impl ::redis::ToRedisArgs) -> #ret_type {
                        self.0.arg(v);
                        #ret_val
                    }
                }
            })
        })
        .collect();

    // Generate initial method for chain.
    let init = match script.args().get(0) {
        Some(arg) => {
            let method = to_ident(arg.as_rust());

            let (ret_type, ret_val) = rets(
                script.args().get(1),
                quote! {
                    invoke
                },
            );

            quote! {
                fn #method<'a>(&'a self, v: impl ::redis::ToRedisArgs) -> #ret_type {
                    let mut invoke = self.0.prepare_invoke();
                    invoke.arg(v);
                    #ret_val
                }
            }
        }
        None => {
            quote! {}
        }
    };

    let script = script.wrap();

    let script_code = quote! {
        {
            struct ReusableScript(::redis::Script);

            #(#chain)*

            impl ReusableScript {
                #init
            }

            ReusableScript(::redis::Script::new(#script))
        }
    };
    script_code.into()
}
