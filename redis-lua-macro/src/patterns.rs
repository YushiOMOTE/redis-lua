use crate::{
    proc_macro::{Ident, Span, TokenStream as TokenStream1, TokenTree},
    script::{Arg, ArgType, Script},
};
use proc_macro2::TokenStream;
use quote::quote;

pub fn new_ident(s: String) -> TokenStream {
    let s: TokenStream1 = TokenTree::Ident(Ident::new(&s, Span::call_site())).into();
    s.into()
}

pub fn all(script: &Script) -> impl Iterator<Item = (usize, &Arg)> {
    script.args().iter().enumerate()
}

pub fn caps(script: &Script) -> impl Iterator<Item = (usize, &Arg)> {
    all(script).filter(|(_, arg)| arg.atype() == ArgType::Cap)
}

pub fn vars(script: &Script) -> impl Iterator<Item = (usize, &Arg)> {
    all(script).filter(|(_, arg)| arg.atype() == ArgType::Var)
}

pub fn to_name((_index, arg): (usize, &Arg)) -> TokenStream {
    new_ident(arg.as_rust().to_string())
}

pub fn to_type((index, _): (usize, &Arg)) -> TokenStream {
    new_ident(format!("A{}", index))
}

pub fn to_param((index, _): (usize, &Arg)) -> TokenStream {
    new_ident(format!("a{}", index))
}

pub fn to_mem(t: (usize, &Arg)) -> TokenStream {
    let p = to_param(t);
    let t = to_type(t);
    quote! { #p: Option<#t> }
}

pub fn to_arg(t: (usize, &Arg)) -> TokenStream {
    let p = to_param(t);
    let t = to_type(t);
    quote! { #p: #t }
}

pub fn to_invoke(t: (usize, &Arg)) -> TokenStream {
    let p = to_param(t);
    quote! {
        invoke.arg(self.#p.take().unwrap())
    }
}

pub fn to_bound(t: (usize, &Arg)) -> TokenStream {
    let t = to_type(t);
    quote! { #t: redis_lua::redis::ToRedisArgs }
}

pub fn varlen(script: &Script) -> usize {
    vars(script).count()
}
