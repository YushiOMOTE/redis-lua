use crate::{patterns::*, script::Script};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Clone, Debug)]
pub struct Chain<'a> {
    index: usize,
    script: &'a Script,
}

impl<'a> Chain<'a> {
    fn new(index: usize, script: &'a Script) -> Self {
        Self { index, script }
    }

    fn next(&self) -> Option<Chain<'a>> {
        let next_index = self.index + 1;
        if next_index <= varlen(self.script) {
            Some(Chain::new(next_index, self.script))
        } else {
            None
        }
    }

    fn prev(&self) -> Option<Chain<'a>> {
        if self.index == 0 {
            None
        } else {
            Some(Chain::new(self.index - 1, self.script))
        }
    }

    fn pchain(&self) -> Option<PartialChain<'a>> {
        if self.index < varlen(self.script) {
            Some(PartialChain {
                chain: self.clone(),
            })
        } else {
            None
        }
    }

    // ChainN
    fn name(&self) -> TokenStream {
        new_ident(format!("Chain{}", self.index))
    }

    fn impl_ctors(&self) -> TokenStream {
        if let Some(prev) = self.prev() {
            // If there's a previous chain,
            let name = prev.tyname();
            let varparam = prev.varparam();
            let vartype = prev.vartype();
            let inits = prev.params_f(|a| quote! { #a: prev.#a });

            quote! {
                fn chain(prev: #name, var: #vartype) -> Self {
                    Self {
                        info: prev.info,
                        inner: prev.inner,
                        #(#inits,)*
                        #varparam: Some(var),
                    }
                }
            }
        } else {
            let args = self.args();
            let inits_new = self.params_f(|a| quote! { #a: Some(#a) });

            quote! {
                fn new(info: redis_lua::Info, inner: I, #(#args),*) -> Self {
                    Self {
                        info,
                        inner,
                        #(#inits_new,)*
                    }
                }
            }
        }
    }

    fn impl_methods(&self) -> TokenStream {
        if let Some(next) = self.next() {
            let name = next.name();
            let tyname = next.tyname();
            let varname = self.varname();
            let vartype = self.vartype();

            quote! {
                fn #varname<#vartype>(self, var: #vartype) -> #tyname {
                    #name::chain(self, var)
                }
            }
        } else {
            let bounds = self.bounds();

            quote! {
                fn invoke<T>(self, con: &mut dyn redis_lua::redis::ConnectionLike) -> redis_lua::redis::RedisResult<T>
                where
                    T: redis_lua::redis::FromRedisValue,
                    I: redis_lua::Script,
                    Self: Sized,
                    #(#bounds),*
                {
                    redis_lua::Script::invoke(self, con)
                }

                fn invoke_async<C, T>(self, con: C) -> redis_lua::redis::RedisFuture<(C, T)>
                where
                    C: redis_lua::redis::aio::ConnectionLike + Clone + Send + 'static,
                    T: redis_lua::redis::FromRedisValue + Send + 'static,
                    I: redis_lua::Script,
                    Self: Sized,
                    #(#bounds),*
                {
                    redis_lua::Script::invoke_async(self, con)
                }
            }
        }
    }

    fn impl_adders(&self) -> TokenStream {
        if self.prev().is_some() {
            return quote! {};
        }

        let tyname = self.tyname();
        let name = self.name();
        let types = self.types();
        let inits_self = self.params_f(|a| quote! { #a: self.#a });

        let impl_takeunit = quote! {
            impl<I, I2, #(#types),*> redis_lua::TakeScript<I2> for #tyname
            where
                I: redis_lua::Script,
                I2: redis_lua::Script,
            {
                type Item = #name<redis_lua::ScriptJoin<I2, I>, #(#types),*>;

                fn take(self, inner: I2) -> Self::Item {
                    #name {
                        inner: inner.join(self.inner),
                        info: self.info,
                        #(#inits_self,)*
                    }
                }
            }
        };

        let impl_add = if let Some(pchain) = self.pchain() {
            let pname = pchain.name();

            quote! {
                impl<I, S, #(#types),*> std::ops::Add<S> for #tyname {
                    type Output = #pname<I, S, #(#types),*>;

                    fn add(self, other: S) -> Self::Output {
                        #pname::new(self, other)
                    }
                }
            }
        } else {
            quote! {
                impl<I, S, #(#types),*> std::ops::Add<S> for #tyname
                where
                    S: redis_lua::TakeScript<#tyname>,
                {
                    type Output = S::Item;

                    fn add(self, other: S) -> Self::Output {
                        other.take(self)
                    }
                }
            }
        };

        quote! {
            #impl_takeunit

            #impl_add
        }
    }

    fn impl_script(&self) -> TokenStream {
        if self.next().is_some() {
            return quote! {};
        }

        let tyname = self.tyname();
        let types = self.types();
        let bounds = self.bounds();
        let invokes = self.invokes();

        quote! {
            impl<I, #(#types),*> redis_lua::Script for #tyname
            where
                I: redis_lua::Script,
                #(#bounds,)*
            {
                fn apply(&mut self, invoke: &mut redis_lua::redis::ScriptInvocation) {
                    self.inner.apply(invoke);
                    #(#invokes;)*
                }

                fn info(&self, info: &mut Vec<redis_lua::Info>) {
                    self.inner.info(info);
                    info.push(self.info.clone());
                }
            }
        }
    }

    pub fn gen(&self) -> TokenStream {
        let tyname = self.tyname();
        let types = self.types();
        let mems = self.mems();

        let impl_ctors = self.impl_ctors();
        let impl_methods = self.impl_methods();
        let impl_adders = self.impl_adders();
        let impl_script = self.impl_script();

        quote! {
            #[derive(Clone, Debug)]
            struct #tyname {
                info: redis_lua::Info,
                inner: I,
                #(#mems,)*
            }

            impl<I, #(#types),*> #tyname
            {
                #impl_ctors

                #impl_methods
            }

            #impl_script

            #impl_adders
        }
    }

    // ChainM<I, A1, A2, ...>
    fn tyname(&self) -> TokenStream {
        let name = self.name();
        let types = self.types();

        quote! {
            #name<I, #(#types),*>
        }
    }

    // A0, A1, A2, ...
    fn types(&self) -> Vec<TokenStream> {
        caps(self.script)
            .map(to_type)
            .chain(vars(self.script).map(to_type).take(self.index))
            .collect()
    }

    // a0, a1, a2, ...
    fn params(&self) -> Vec<TokenStream> {
        caps(self.script)
            .map(to_param)
            .chain(vars(self.script).map(to_param).take(self.index))
            .collect()
    }

    fn params_f<F>(&self, map: F) -> Vec<TokenStream>
    where
        F: Fn(TokenStream) -> TokenStream,
    {
        self.params().into_iter().map(map).collect()
    }

    // a0: A0, a1: A1, a2: A2, ...
    fn args(&self) -> Vec<TokenStream> {
        caps(self.script)
            .map(to_arg)
            .chain(vars(self.script).map(to_arg).take(self.index))
            .collect()
    }

    // a0: A0, a1: A1, a2: A2, ...
    fn mems(&self) -> Vec<TokenStream> {
        caps(self.script)
            .map(to_mem)
            .chain(vars(self.script).map(to_mem).take(self.index))
            .collect()
    }

    // `x`
    fn varname(&self) -> TokenStream {
        vars(self.script).map(to_name).nth(self.index).unwrap()
    }

    // `a3`
    fn varparam(&self) -> TokenStream {
        vars(self.script).map(to_param).nth(self.index).unwrap()
    }

    // `A3`
    fn vartype(&self) -> TokenStream {
        vars(self.script).map(to_type).nth(self.index).unwrap()
    }

    // `A0: redis_lua::ToRedisArgs`, ...
    fn bounds(&self) -> Vec<TokenStream> {
        caps(self.script)
            .map(to_bound)
            .chain(vars(self.script).map(to_bound).take(self.index))
            .collect()
    }

    // `A0: redis_lua::ToRedisArgs`, ...
    fn invokes(&self) -> Vec<TokenStream> {
        all(self.script).map(to_invoke).collect()
    }
}

#[derive(Clone, Debug)]
pub struct PartialChain<'a> {
    chain: Chain<'a>,
}

impl<'a> PartialChain<'a> {
    fn new(chain: Chain<'a>) -> Self {
        PartialChain { chain }
    }

    fn next(&self) -> Option<PartialChain<'a>> {
        self.chain.next().and_then(|c| {
            // PartialChain exists only if the corresponding chain is not the last one.
            c.next()?;
            Some(Self::new(c))
        })
    }

    fn impl_methods(&self) -> TokenStream {
        let varname = self.chain.varname();
        let vartype = self.chain.vartype();

        if let Some(next) = self.next() {
            let name = next.name();
            let tyname = next.tyname();

            quote! {
                fn #varname<#vartype>(self, var: #vartype) -> #tyname {
                    #name::new(self.chain.#varname(var), self.next)
                }
            }
        } else {
            let tyname = self.chain.next().unwrap().tyname();

            quote! {
                fn #varname<#vartype>(self, var: #vartype) -> S::Item
                where
                    S: redis_lua::TakeScript<#tyname>,
                    #vartype: redis_lua::redis::ToRedisArgs,
                {
                    let chain = self.chain.#varname(var);
                    let next = self.next;
                    next.take(chain)
                }
            }
        }
    }

    fn impl_adders(&self) -> TokenStream {
        if self.chain.prev().is_some() {
            return quote! {};
        }

        let name = self.name();
        let types = self.chain.types();

        quote! {
            impl<I, I2, S, #(#types),*> redis_lua::TakeScript<I> for #name<I2, S, #(#types),*>
            where
                I: redis_lua::Script,
                I2: redis_lua::Script,
            {
                type Item = #name<redis_lua::ScriptJoin<I, I2>, S, #(#types),*>;

                fn take(self, inner: I) -> Self::Item {
                    Self::Item::new(self.chain.take(inner), self.next)
                }
            }

            impl<I, S2, S1, #(#types),*> std::ops::Add<S2> for #name<I, S1, #(#types),*>
            where
                S1: std::ops::Add<S2>,
            {
                type Output = #name<I, S1::Output, #(#types),*>;

                fn add(self, other: S2) -> Self::Output {
                    #name::new(self.chain, self.next + other)
                }
            }
        }
    }

    pub fn gen(&self) -> TokenStream {
        let tyname = self.tyname();
        let chain_tyname = self.chain.tyname();
        let types = self.chain.types();

        let impl_methods = self.impl_methods();
        let impl_adders = self.impl_adders();

        quote! {
            #[derive(Clone, Debug)]
            struct #tyname {
                chain: #chain_tyname,
                next: S,
            }

            impl<I, S, #(#types),*> #tyname {
                fn new(chain: #chain_tyname, next: S) -> Self {
                    Self { chain, next }
                }

                #impl_methods
            }

            #impl_adders
        }
    }

    // PartialChainM<I, S, A1, A2, ...>
    fn tyname(&self) -> TokenStream {
        let name = self.name();
        let types = self.chain.types();

        quote! {
            #name<I, S, #(#types),*>
        }
    }

    fn name(&self) -> TokenStream {
        new_ident(format!("PartialChain{}", self.chain.index))
    }
}

pub struct ChainIter<'a> {
    chain: Option<Chain<'a>>,
}

impl<'a> ChainIter<'a> {
    pub fn new(script: &'a Script) -> Self {
        Self {
            chain: Some(Chain::new(0, script)),
        }
    }

    pub fn pchains(&self) -> PartialChainIter<'a> {
        PartialChainIter {
            pchain: self.chain.clone().and_then(|p| p.pchain()),
        }
    }
}

impl<'a> Iterator for ChainIter<'a> {
    type Item = Chain<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let chain = self.chain.take();
        let ret = chain?;
        self.chain = ret.next();
        Some(ret)
    }
}

pub struct PartialChainIter<'a> {
    pchain: Option<PartialChain<'a>>,
}

impl<'a> Iterator for PartialChainIter<'a> {
    type Item = PartialChain<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let pchain = self.pchain.take();
        let ret = pchain?;
        self.pchain = ret.next();
        Some(ret)
    }
}
