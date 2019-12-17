use crate::{
    proc_macro::{Span, TokenStream, TokenTree},
    token::{retokenize, Pos, Token},
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgType {
    Cap,
    Var,
}

#[derive(Debug, Clone)]
pub struct Arg {
    key: Token,
    rust: TokenTree,
    lua: String,
    argv: String,
    atype: ArgType,
}

impl Arg {
    fn new(key: Token, rust: TokenTree, lua: String, argv: String, atype: ArgType) -> Self {
        Self {
            key,
            rust,
            lua,
            argv,
            atype,
        }
    }

    /// Token string inside `lua!`
    pub fn key(&self) -> &Token {
        &self.key
    }

    /// As rust variable, e.g. `x`
    pub fn as_rust(&self) -> &TokenTree {
        &self.rust
    }

    /// As lua internal variable, e.g. `__internal_from_args1`
    pub fn as_lua(&self) -> &str {
        &self.lua
    }

    /// As `ARGV` parameter, e.g. `ARGV[1]`
    pub fn as_argv(&self) -> &str {
        &self.argv
    }

    pub fn atype(&self) -> ArgType {
        self.atype
    }
}

#[derive(Debug)]
pub struct Args(Vec<Arg>);

impl Args {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, token: &Token) -> Arg {
        let tt = token.tree();
        let key = token.clone();

        match self.0.iter().find(|arg| arg.key() == &key) {
            Some(arg) => arg.clone(),
            None => {
                let rust = tt.clone();
                let lua = format!("__internal_from_args_{}", self.0.len());
                let argv = format!("ARGV[{}]", self.0.len() + 1);
                let atype = if token.is_cap() {
                    ArgType::Cap
                } else {
                    ArgType::Var
                };

                let arg = Arg::new(key, rust, lua, argv, atype);
                self.0.push(arg.clone());
                arg
            }
        }
    }

    pub fn args(&self) -> &[Arg] {
        &self.0
    }
}

#[derive(Debug)]
pub struct Script {
    script: String,
    wrapped: String,
    spans: BTreeMap<usize, Span>,
    args: Args,
}

impl Script {
    pub fn new(tokens: TokenStream, convert_args: bool) -> Self {
        let tokens = retokenize(tokens);

        // Script string to be checked & emitted.
        let mut script = String::new();

        // Table to map lua code span to rust code span.
        let mut spans = BTreeMap::new();

        // Script argument lists (i.e. `ARGV`).
        let mut args = Args::new();

        let mut pos = Option::<Pos>::None;

        for t in tokens {
            let (code, span) = if t.is_arg() && convert_args {
                let arg = args.add(&t);
                (arg.as_lua().into(), t.span())
            } else {
                (t.to_string(), t.span())
            };

            // match t.arg() {
            //     Some(t) if convert_args => {
            //         // Script argument like `@ident` is converted to
            //         // a special variable like `__internal_0` in lua.

            //         let arg = args.add(t.tree());
            //         (arg.as_lua().into(), t.span())
            //     }
            //     _ => (t.to_string(), t.span()),
            // };

            let (line, col) = (t.start().line, t.start().column);
            let (prev_line, prev_col) = pos
                .take()
                .map(|lc| (lc.line, lc.column))
                .unwrap_or_else(|| (line, col));

            if line > prev_line {
                script.push_str("\n");
            } else if line == prev_line {
                for _ in 0..col.saturating_sub(prev_col) {
                    script.push_str(" ");
                }
            }
            let begin = script.len();
            script.push_str(&code.to_string());
            let end = script.len();

            for i in begin..=end {
                spans.insert(i, span.clone());
            }

            pos = Some(t.end());
        }

        let script = script.trim_end().to_string();

        // `wrapped` contains `script` plus variable initialization logic at the top.
        // Only `script` part is checked by the linter. The linter is configured
        // so that it allows only special local variables like `__internal_0` but doesn't
        // allow `ARGV`. This is to prevent script authers from accidentally writing
        // `ARGV[x]` where `x` is larger than actual arguments given by a command.
        let wrapped = if convert_args {
            let wrapper = args.args().iter().fold(String::new(), |s, arg| {
                // Generating these lines.
                //
                // ```
                // local __internal_0 = ARGV[0];
                // local __internal_1 = ARGV[1];
                // local __internal_2 = ARGV[2];
                // ```
                s + &format!("local {} = {}; ", arg.as_lua(), arg.as_argv())
            });
            format!("{}\n{}", wrapper, script)
        } else {
            "".into()
        };

        // println!("--- Lua script ---\n{}\n-------", script);

        Self {
            script,
            wrapped,
            spans,
            args,
        }
    }

    pub fn script(&self) -> &str {
        &self.script
    }

    pub fn wrap(&self) -> &str {
        &self.wrapped
    }

    pub fn args(&self) -> &[Arg] {
        self.args.args()
    }

    /// Convert lua code span to rust code span.
    pub fn range_to_span(&self, range: (usize, usize)) -> Vec<Span> {
        self.spans
            .range(range.0..=range.1)
            .map(|(_, v)| v.clone())
            .collect()
    }
}
