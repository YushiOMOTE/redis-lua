use itertools::Itertools;
use proc_macro::{Delimiter, Span, TokenStream, TokenTree};
use std::{
    fmt::{self, Display, Formatter},
    iter::IntoIterator,
    vec::IntoIter,
};

#[derive(Clone)]
pub struct Token(String, TokenTree);

impl Token {
    pub fn tree(&self) -> &TokenTree {
        &self.1
    }

    pub fn arg(&self) -> Option<Token> {
        if self.0.starts_with("@") {
            Some(self.rename(&self.0[1..]))
        } else {
            None
        }
    }

    pub fn span(&self) -> Span {
        self.1.span()
    }

    fn is(&self, s: &str) -> bool {
        self.0 == s
    }

    fn is_marker(&self) -> bool {
        self.0 == "@"
    }

    fn rename(&self, s: &str) -> Self {
        Token(s.into(), self.1.clone())
    }
}

pub struct Tokens(Vec<Token>);

pub fn retokenize(tt: TokenStream) -> Tokens {
    Tokens(
        tt.into_iter()
            .map(|tt| Tokens::from(tt))
            .flatten()
            .peekable()
            .batching(|iter| {
                let t = iter.next()?;

                if t.is_marker() {
                    // `@` + `ident` => `@ident`
                    let t = iter.next().expect("@ must trail an identifier");
                    Some(t.rename(&format!("@{}", t)))
                } else if t.is(".") && iter.peek().map(|t| t.is(".")).unwrap_or(false) {
                    // `.` + `.` => `..`
                    iter.next().unwrap();
                    Some(t.rename(".."))
                } else if t.is("=") && iter.peek().map(|t| t.is("=")).unwrap_or(false) {
                    // `=` + `=` => `==`
                    iter.next().unwrap();
                    Some(t.rename("=="))
                } else {
                    Some(t)
                }
            })
            .collect(),
    )
}

fn delimiter(d: Delimiter) -> (String, String) {
    let (b, e) = match d {
        Delimiter::Parenthesis => ("(", ")"),
        Delimiter::Brace => ("{", "}"),
        Delimiter::Bracket => ("[", "]"),
        Delimiter::None => ("", ""),
    };
    (b.into(), e.into())
}

impl IntoIterator for Tokens {
    type Item = Token;
    type IntoIter = IntoIter<Token>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<TokenTree> for Tokens {
    fn from(tt: TokenTree) -> Self {
        let tts = match tt.clone() {
            TokenTree::Group(g) => {
                let (b, e) = delimiter(g.delimiter());

                vec![Token(b, tt.clone())]
                    .into_iter()
                    .chain(g.stream().into_iter().map(|tt| Tokens::from(tt)).flatten())
                    .chain(vec![Token(e, tt.clone())])
                    .collect()
            }
            _ => vec![Token(tt.to_string(), tt)],
        };
        Tokens(tts)
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
