use crate::{
    proc_macro::{Span, TokenStream, TokenTree},
    token::retokenize,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Arg {
    key: String,
    rust: TokenTree,
    lua: String,
    argv: String,
}

impl Arg {
    fn new(key: String, rust: TokenTree, lua: String, argv: String) -> Self {
        Self {
            key,
            rust,
            lua,
            argv,
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn as_rust(&self) -> &TokenTree {
        &self.rust
    }

    pub fn as_lua(&self) -> &str {
        &self.lua
    }

    pub fn as_argv(&self) -> &str {
        &self.argv
    }
}

#[derive(Debug)]
pub struct Args(Vec<Arg>);

impl Args {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, token: &TokenTree) -> Arg {
        let key = token.to_string();

        match self.0.iter().find(|arg| arg.key() == &key) {
            Some(arg) => arg.clone(),
            None => {
                let rust = token.clone();
                let lua = format!("__internal_from_args_{}", self.0.len());
                let argv = format!("ARGV[{}]", self.0.len() + 1);

                let arg = Arg::new(key, rust, lua, argv);
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

        for t in tokens {
            let (code, span) = match t.arg() {
                Some(t) if convert_args => {
                    // Script argument like `@ident` is converted to
                    // a special variable like `__internal_0` in lua.

                    let arg = args.add(t.tree());
                    (arg.as_lua().into(), t.span())
                }
                _ => (t.to_string(), t.span()),
            };

            let begin = script.len();
            script.push_str(&code.to_string());
            script.push_str(" ");
            let end = script.len();
            for i in begin..=end {
                spans.insert(i, span.clone());
            }
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

        println!("Lua script: {}", script);

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
