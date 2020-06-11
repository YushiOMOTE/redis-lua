use itertools::Itertools;
use proc_macro::{Delimiter, Span, TokenStream, TokenTree};
use proc_macro2::Span as Span2;
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
    fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

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

fn span_pos(span: &Span) -> (Pos, Pos) {
    let span2: Span2 = span.clone().into();
    let start = span2.start();
    let end = span2.end();

    // In stable, line/column information is not provided
    // and set to 0 (line is 1-indexed)
    if start.line == 0 || end.line == 0 {
        return fallback_span_pos(span);
    }

    (
        Pos::new(start.line, start.column),
        Pos::new(end.line, end.column),
    )
}

fn parse_pos(span: &Span) -> Option<(usize, usize)> {
    // Workaround to somehow retrieve location information in span in stable rust :(

    let re = regex::Regex::new(r"bytes\(([0-9]+)\.\.([0-9]+)\)").unwrap();
    match re.captures(&format!("{:?}", span)) {
        Some(caps) => match (caps.get(1), caps.get(2)) {
            (Some(start), Some(end)) => Some((
                match start.as_str().parse() {
                    Ok(v) => v,
                    _ => return None,
                },
                match end.as_str().parse() {
                    Ok(v) => v,
                    _ => return None,
                },
            )),
            _ => None,
        },
        None => None,
    }
}

fn fallback_span_pos(span: &Span) -> (Pos, Pos) {
    let (start, end) = match parse_pos(span) {
        Some(v) => v,
        None => proc_macro_error::abort_call_site!(
            "Cannot retrieve span information; please use nightly"
        ),
    };
    (Pos::new(1, start), Pos::new(1, end))
}

/// Attribute of token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TokenAttr {
    /// No attribute
    None,
    /// Starts with `$`
    Var,
    /// Starts with `@`
    Cap,
}

#[derive(Clone, Debug)]
pub struct Token {
    source: String,
    tree: TokenTree,
    start: Pos,
    end: Pos,
    attr: TokenAttr,
}

impl std::cmp::PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source && self.attr == other.attr
    }
}

impl std::cmp::Eq for Token {}

impl Token {
    fn new(tree: TokenTree) -> Self {
        let (start, end) = span_pos(&tree.span());
        Self {
            source: tree.to_string(),
            start,
            end,
            tree,
            attr: TokenAttr::None,
        }
    }

    fn new_delim(source: String, tree: TokenTree, open: bool) -> Self {
        let (start, end) = span_pos(&tree.span());
        let (start, end) = if open {
            (start, start.right())
        } else {
            (end.left(), end)
        };

        Self {
            source,
            tree,
            start,
            end,
            attr: TokenAttr::None,
        }
    }

    pub fn tree(&self) -> &TokenTree {
        &self.tree
    }

    pub fn is_arg(&self) -> bool {
        self.is_var() || self.is_cap()
    }

    pub fn is_var(&self) -> bool {
        self.attr == TokenAttr::Var
    }

    pub fn is_cap(&self) -> bool {
        self.attr == TokenAttr::Cap
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

    fn is(&self, s: &str) -> bool {
        self.source == s
    }

    fn attr(mut self, attr: TokenAttr) -> Self {
        self.attr = attr;
        self
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
                // Find variable/capture tokens
                let t = iter.next()?;
                if t.is("@") {
                    // `@` + `ident` => `@ident`
                    let t = iter.next().expect("@ must trail an identifier");
                    Some(t.attr(TokenAttr::Cap))
                } else if t.is("$") {
                    // `$` + `ident` => `@ident`
                    let t = iter.next().expect("$ must trail an identifier");
                    Some(t.attr(TokenAttr::Var))
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
            _ => vec![Token::new(tt)],
        };
        Tokens(tts)
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.source)
    }
}
