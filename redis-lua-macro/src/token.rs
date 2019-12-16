use itertools::Itertools;
use proc_macro::{Delimiter, LineColumn, Span, TokenStream, TokenTree};
use std::{
    fmt::{self, Display, Formatter},
    iter::IntoIterator,
    vec::IntoIter,
};

#[derive(Clone, Copy, Debug)]
pub struct Pos {
    pub line: usize,
    pub column: usize,
}

impl Pos {
    fn left(&self) -> Self {
        Self {
            line: self.line,
            column: self.column.saturating_sub(1),
        }
    }

    fn right(&self) -> Self {
        Self {
            line: self.line,
            column: self.column.saturating_add(1),
        }
    }
}

impl From<LineColumn> for Pos {
    fn from(v: LineColumn) -> Self {
        Pos {
            line: v.line,
            column: v.column,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Token {
    source: String,
    tree: TokenTree,
    start: Pos,
    end: Pos,
}

impl Token {
    fn new(source: String, tree: TokenTree) -> Self {
        Self {
            source,
            start: tree.span().start().into(),
            end: tree.span().end().into(),
            tree,
        }
    }

    fn new_delim(source: String, tree: TokenTree, open: bool) -> Self {
        let (start, end) = if open {
            let pos: Pos = tree.span().start().into();
            (pos, pos.right())
        } else {
            let pos: Pos = tree.span().end().into();
            (pos.left(), pos)
        };

        Self {
            source,
            tree,
            start,
            end,
        }
    }

    pub fn tree(&self) -> &TokenTree {
        &self.tree
    }

    pub fn arg(&self) -> Option<Token> {
        if self.source.starts_with("@") {
            Some(self.rename(&self.source[1..]))
        } else {
            None
        }
    }

    pub fn span(&self) -> Span {
        self.tree.span()
    }

    pub fn start(&self) -> Pos {
        self.start
    }

    pub fn end(&self) -> Pos {
        self.end
    }

    fn is_marker(&self) -> bool {
        self.source == "@"
    }

    fn rename(&self, s: &str) -> Self {
        Token::new(s.into(), self.tree.clone())
    }
}

#[derive(Debug)]
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

                vec![Token::new_delim(b, tt.clone(), true)]
                    .into_iter()
                    .chain(g.stream().into_iter().map(|tt| Tokens::from(tt)).flatten())
                    .chain(vec![Token::new_delim(e, tt.clone(), false)])
                    .collect()
            }
            _ => vec![Token::new(tt.to_string(), tt)],
        };
        Tokens(tts)
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.source)
    }
}
