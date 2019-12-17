#![feature(proc_macro_diagnostic)]
#![feature(proc_macro_span)]

extern crate proc_macro;

use self::proc_macro::{Ident, Span, TokenStream, TokenTree};
use quote::quote;

mod check;
mod file;
mod script;
mod token;

use crate::{
    check::Checker,
    script::{Arg, ArgType, Script},
};

use proc_macro_hack::proc_macro_hack;

fn new_ident(s: String) -> proc_macro2::TokenStream {
    let s: TokenStream = TokenTree::Ident(Ident::new(&s, Span::call_site())).into();
    s.into()
}

fn to_ident(tt: &TokenTree) -> proc_macro2::TokenStream {
    let s: TokenStream = tt.clone().into();
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

fn gen_pchain(script: &Script, index: usize, last: bool) -> proc_macro2::TokenStream {
    let chain = chain_name(index);
    let pchain = pchain_name(index);
    let types = chain_types(script, index);
    let inits = chain_inits(script, quote! { self.chain }, index);

    // Generate methods
    let methods = if last {
        let next_chain = chain_name(index + 1);
        let varname = nth_varname(script, index);
        let vartype = nth_vartype(script, index);

        quote! {
            fn #varname<#vartype>(self, var: #vartype) -> S::Item
            where
                S: redis_lua::TakeScript<#next_chain<I, #(#types,)* #vartype>>,
                #vartype: redis::ToRedisArgs,
            {
                let chain = self.chain.#varname(var);
                let next = self.next;
                next.take(chain)
            }
        }
    } else {
        let next_pchain = pchain_name(index + 1);
        let varname = nth_varname(script, index);
        let vartype = nth_vartype(script, index);

        quote! {
            fn #varname<#vartype>(self, var: #vartype) -> #next_pchain<I, S, #(#types,)* #vartype> {
                #next_pchain {
                    chain: self.chain.#varname(var),
                    next: self.next,
                }
            }
        }
    };

    let impl_takeunit = if index == 0 {
        quote! {
            impl<I, I2, S, #(#types),*> redis_lua::TakeScript<I> for #pchain<I2, S, #(#types),*>
            where
                I: redis_lua::Script,
                I2: redis_lua::Script,
            {
                type Item = #pchain<redis_lua::ScriptJoin<I, I2>, S, #(#types),*>;

                fn take(self, inner: I) -> Self::Item {
                    Self::Item {
                        chain: #chain {
                            info: self.chain.info,
                            inner: inner.join(self.chain.inner),
                            #(#inits,)*
                        },
                        next: self.next,
                    }
                }
            }
        }
    } else {
        quote! {}
    };

    let impl_add = if index == 0 {
        quote! {
            impl<I, S2, S1, #(#types),*> std::ops::Add<S2> for #pchain<I, S1, #(#types),*>
            where
                S1: std::ops::Add<S2>,
            {
                type Output = #pchain<I, S1::Output, #(#types),*>;

                fn add(self, other: S2) -> Self::Output {
                    #pchain::new(self.chain, self.next + other)
                }
            }
        }
    } else {
        quote! {}
    };

    // Generate struct
    quote! {
        #[derive(Clone, Debug)]
        struct #pchain<I, S, #(#types),*> {
            chain: #chain<I, #(#types),*>,
            next: S,
        }

        impl<I, S, #(#types),*> #pchain<I, S, #(#types),*> {
            fn new(chain: #chain<I, #(#types),*>, next: S) -> Self {
                Self { chain, next }
            }

            #methods
        }

        #impl_takeunit

        #impl_add
    }
}

fn gen_chain(script: &Script, index: usize, last: bool) -> proc_macro2::TokenStream {
    let chain = chain_name(index);
    let types = chain_types(script, index);
    let mems = chain_mems(script, index);
    let params = chain_params(script, index);
    let inits = chain_inits(script, quote! { self }, index);

    // Generate methods
    let methods = if last {
        quote! {
            fn invoke<T>(self, con: &mut dyn redis::ConnectionLike) -> redis::RedisResult<T>
            where
                T: redis::FromRedisValue,
            {
                redis_lua::Script::invoke(self, con)
            }

            fn invoke_async<C, T>(self, con: C) -> redis::RedisFuture<(C, T)>
            where
                C: redis::aio::ConnectionLike + Clone + Send + 'static,
                T: redis::FromRedisValue + Send + 'static,
            {
                redis_lua::Script::invoke_async(self, con)
            }
        }
    } else {
        let next_chain = chain_name(index + 1);
        let varname = nth_varname(script, index);
        let varparam = nth_varparam(script, index);
        let vartype = nth_vartype(script, index);

        quote! {
            fn #varname<#vartype>(self, var: #vartype) -> #next_chain<I, #(#types,)* #vartype> {
                #next_chain {
                    info: self.info,
                    inner: self.inner,
                    #(#inits,)*
                    #varparam: var,
                }
            }
        }
    };

    // Generate take unit
    let impl_takeunit = if index == 0 {
        quote! {
            impl<I, I2, #(#types),*> redis_lua::TakeScript<I> for #chain<I2, #(#types),*>
            where
                I: redis_lua::Script,
                I2: redis_lua::Script,
            {
                type Item = #chain<redis_lua::ScriptJoin<I, I2>, #(#types),*>;

                fn take(self, inner: I) -> Self::Item {
                    Self::Item {
                        info: self.info,
                        inner: inner.join(self.inner),
                        #(#inits,)*
                    }
                }
            }
        }
    } else {
        quote! {}
    };

    let impl_add = if index == 0 && last {
        quote! {
            impl<I, S, #(#types),*> std::ops::Add<S> for #chain<I, #(#types),*>
            where
                S: redis_lua::TakeScript<#chain<I, #(#types),*>>,
            {
                type Output = S::Item;

                fn add(self, other: S) -> Self::Output {
                    other.take(self)
                }
            }
        }
    } else if index == 0 && !last {
        let pchain = pchain_name(index);

        quote! {
            impl<I, S, #(#types),*> std::ops::Add<S> for #chain<I, #(#types),*> {
                type Output = #pchain<I, S, #(#types),*>;

                fn add(self, other: S) -> Self::Output {
                    #pchain::new(self, other)
                }
            }
        }
    } else {
        quote! {}
    };

    let bounds_if = if last {
        let bounds = all(script).map(to_bound);
        quote! {
            where
                I: redis_lua::Script,
                #(#bounds,)*
        }
    } else {
        quote! {}
    };

    // Generate unit
    let impl_unit = if last {
        let bounds = all(script).map(to_bound);
        let invokes = all(script).map(to_invoke);

        quote! {
            impl<I, #(#types),*> redis_lua::Script for #chain<I, #(#types),*>
            where
                I: redis_lua::Script,
                #(#bounds,)*
            {
                fn apply(self, invoke: &mut redis::ScriptInvocation) {
                    self.inner.apply(invoke);
                    #(#invokes;)*
                }

                fn info(&self, info: &mut Vec<redis_lua::Info>) {
                    self.inner.info(info);
                    info.push(self.info.clone());
                }
            }
        }
    } else {
        quote! {}
    };

    quote! {
        #[derive(Clone, Debug)]
        struct #chain<I, #(#types),*> {
            info: redis_lua::Info,
            inner: I,
            #(#mems,)*
        }

        impl<I, #(#types),*> #chain<I, #(#types),*>
        #bounds_if
        {
            fn new(info: redis_lua::Info, inner: I, #(#mems),*) -> Self {
                Self {
                    info,
                    inner,
                    #(#params,)*
                }
            }

            #methods
        }

        #impl_unit

        #impl_takeunit

        #impl_add
    }
}

fn gen_all(script: &Script) -> proc_macro2::TokenStream {
    let len = varlen(script) + 1;
    (0..len).fold(proc_macro2::TokenStream::new(), |mut ts, i| {
        ts.extend(gen_chain(script, i, i == len - 1));
        if len >= 2 && i <= len - 2 {
            ts.extend(gen_pchain(script, i, i == len - 2));
        }
        ts
    })
}

fn chain_name(index: usize) -> proc_macro2::TokenStream {
    new_ident(format!("Chain{}", index))
}

fn pchain_name(index: usize) -> proc_macro2::TokenStream {
    new_ident(format!("PartialChain{}", index))
}

fn chain_types(script: &Script, index: usize) -> Vec<proc_macro2::TokenStream> {
    caps(script)
        .map(to_type)
        .chain(vars(script).map(to_type).take(index))
        .collect()
}

fn chain_mems(script: &Script, index: usize) -> Vec<proc_macro2::TokenStream> {
    caps(script)
        .map(to_mem)
        .chain(vars(script).map(to_mem).take(index))
        .collect()
}

fn chain_params(script: &Script, index: usize) -> Vec<proc_macro2::TokenStream> {
    caps(script)
        .map(to_param)
        .chain(vars(script).map(to_param).take(index))
        .collect()
}

fn chain_inits(
    script: &Script,
    pfx: proc_macro2::TokenStream,
    index: usize,
) -> Vec<proc_macro2::TokenStream> {
    caps(script)
        .map(|t| to_init(&pfx, t))
        .chain(vars(script).map(|t| to_init(&pfx, t)).take(index))
        .collect()
}

fn nth_varname(script: &Script, index: usize) -> proc_macro2::TokenStream {
    vars(script).map(to_name).nth(index).unwrap()
}

fn nth_varparam(script: &Script, index: usize) -> proc_macro2::TokenStream {
    vars(script).map(to_param).nth(index).unwrap()
}

fn nth_vartype(script: &Script, index: usize) -> proc_macro2::TokenStream {
    vars(script).map(to_type).nth(index).unwrap()
}

fn all(script: &Script) -> impl Iterator<Item = (usize, &Arg)> {
    script.args().iter().enumerate()
}

fn caps(script: &Script) -> impl Iterator<Item = (usize, &Arg)> {
    all(script).filter(|(_, arg)| arg.atype() == ArgType::Cap)
}

fn vars(script: &Script) -> impl Iterator<Item = (usize, &Arg)> {
    all(script).filter(|(_, arg)| arg.atype() == ArgType::Var)
}

fn to_name((_index, arg): (usize, &Arg)) -> proc_macro2::TokenStream {
    new_ident(arg.as_rust().to_string())
}

fn to_type((index, _): (usize, &Arg)) -> proc_macro2::TokenStream {
    new_ident(format!("A{}", index))
}

fn to_param((index, _): (usize, &Arg)) -> proc_macro2::TokenStream {
    new_ident(format!("a{}", index))
}

fn to_mem(t: (usize, &Arg)) -> proc_macro2::TokenStream {
    let p = to_param(t);
    let t = to_type(t);
    quote! { #p: #t }
}

fn to_init(pfx: &proc_macro2::TokenStream, t: (usize, &Arg)) -> proc_macro2::TokenStream {
    let p = to_param(t);
    quote! { #p: #pfx.#p }
}

fn to_invoke(t: (usize, &Arg)) -> proc_macro2::TokenStream {
    let p = to_param(t);
    quote! {
        invoke.arg(self.#p)
    }
}

fn to_bound(t: (usize, &Arg)) -> proc_macro2::TokenStream {
    let t = to_type(t);
    quote! { #t: redis::ToRedisArgs }
}

fn varlen(script: &Script) -> usize {
    vars(script).count()
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

    let defs = gen_all(&script);

    let body_str = script.script();
    let script_str = script.wrap();

    let args = all(&script).map(|(_, arg)| {
        let arg = arg.as_lua().to_string();
        quote! { #arg }
    });

    let caps = caps(&script).map(|(_, arg)| {
        let arg = to_ident(arg.as_rust());
        quote! {
            #arg
        }
    });

    let script_code = quote! {
        {
            use redis_lua::Script;

            #defs

            Chain0::new(redis_lua::Info::new(#script_str, #body_str, &[#(#args),*]), (), #(#caps),*)
        }
    };
    script_code.into()
}
