//! Tokenization and parsing of source code.

use std::fmt::{self, Display, Formatter};
use unicode_xid::UnicodeXID;

use crate::func::LayoutFunc;
use crate::size::{Size, ScaleSize};

mod tokens;
#[macro_use]
mod parsing;
mod span;

pub use span::{Span, Spanned};
pub use tokens::{tokenize, Tokens};
pub use parsing::{parse, ParseContext, ParseResult};

/// A logical unit of the incoming text stream.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Token<'s> {
    /// One or more whitespace (non-newline) codepoints.
    Space,
    /// A line feed (`\n`, `\r\n` and some more as defined by the Unicode standard).
    Newline,
    /// A left bracket: `[`.
    LeftBracket,
    /// A right bracket: `]`.
    RightBracket,
    /// A colon (`:`) indicating the beginning of function arguments (Function
    /// header only).
    ///
    /// If a colon occurs outside of a function header, it will be tokenized as
    /// [Text](Token::Text), just like the other tokens annotated with
    /// _Header only_.
    Colon,
    /// An equals (`=`) sign assigning a function argument a value (Header only).
    Equals,
    /// A comma (`,`) separating two function arguments (Header only).
    Comma,
    /// Quoted text as a string value (Header only).
    Quoted(&'s str),
    /// An underscore, indicating text in italics (Body only).
    Underscore,
    /// A star, indicating bold text (Body only).
    Star,
    /// A backtick, indicating monospace text (Body only).
    Backtick,
    /// A line comment.
    LineComment(&'s str),
    /// A block comment.
    BlockComment(&'s str),
    /// A star followed by a slash unexpectedly ending a block comment
    /// (the comment was not started before, otherwise a
    /// [BlockComment](Token::BlockComment) would be returned).
    StarSlash,
    /// Any consecutive string which does not contain markup.
    Text(&'s str),
}

/// A tree representation of source code.
#[derive(Debug, PartialEq)]
pub struct SyntaxTree {
    pub nodes: Vec<Spanned<Node>>,
}

impl SyntaxTree {
    /// Create an empty syntax tree.
    pub fn new() -> SyntaxTree {
        SyntaxTree { nodes: vec![] }
    }
}

/// A node in the syntax tree.
#[derive(Debug, PartialEq)]
pub enum Node {
    /// Whitespace.
    Space,
    /// A line feed.
    Newline,
    /// Indicates that italics were enabled / disabled.
    ToggleItalics,
    /// Indicates that boldface was enabled / disabled.
    ToggleBold,
    /// Indicates that monospace was enabled / disabled.
    ToggleMonospace,
    /// Literal text.
    Text(String),
    /// A function invocation.
    Func(FuncCall),
}

/// An invocation of a function.
#[derive(Debug)]
pub struct FuncCall {
    pub call: Box<dyn LayoutFunc>,
}

impl PartialEq for FuncCall {
    fn eq(&self, other: &FuncCall) -> bool {
        &self.call == &other.call
    }
}

/// The arguments passed to a function.
#[derive(Debug, Clone, PartialEq)]
pub struct FuncArgs {
    pub pos: Vec<Spanned<PosArg>>,
    pub key: Vec<Spanned<KeyArg>>,
}

impl FuncArgs {
    /// Create an empty collection of arguments.
    pub fn new() -> FuncArgs {
        FuncArgs {
            pos: vec![],
            key: vec![],
        }
    }

    /// Add a positional argument.
    pub fn add_pos(&mut self, arg: Spanned<PosArg>) {
        self.pos.push(arg);
    }

    /// Add a keyword argument.
    pub fn add_key(&mut self, arg: Spanned<KeyArg>) {
        self.key.push(arg);
    }

    /// Force-extract the first positional argument.
    pub fn get_pos<E: ExpressionKind>(&mut self) -> ParseResult<Spanned<E>> {
        expect(self.get_pos_opt())
    }

    /// Extract the first positional argument.
    pub fn get_pos_opt<E: ExpressionKind>(&mut self) -> ParseResult<Option<Spanned<E>>> {
        Ok(if !self.pos.is_empty() {
            let spanned = self.pos.remove(0);
            let span = spanned.span;
            Some(Spanned::new(E::from_expr(spanned)?, span))
        } else {
            None
        })
    }

    /// Iterator over positional arguments.
    pub fn pos(&mut self) -> std::vec::IntoIter<Spanned<PosArg>> {
        let vec = std::mem::replace(&mut self.pos, vec![]);
        vec.into_iter()
    }

    /// Force-extract a keyword argument.
    pub fn get_key<E: ExpressionKind>(&mut self, name: &str) -> ParseResult<Spanned<E>> {
        expect(self.get_key_opt(name))
    }

    /// Extract a keyword argument.
    pub fn get_key_opt<E: ExpressionKind>(&mut self, name: &str) -> ParseResult<Option<Spanned<E>>> {
        Ok(if let Some(index) = self.key.iter().position(|arg| arg.v.key.v.0 == name) {
            let Spanned { v, span } = self.key.swap_remove(index);
            Some(Spanned::new(E::from_expr(v.value)?, span))
        } else {
            None
        })
    }

    /// Extract any keyword argument.
    pub fn get_key_next(&mut self) -> Option<Spanned<KeyArg>> {
        self.key.pop()
    }

    /// Iterator over all keyword arguments.
    pub fn keys(&mut self) -> std::vec::IntoIter<Spanned<KeyArg>> {
        let vec = std::mem::replace(&mut self.key, vec![]);
        vec.into_iter()
    }

    /// Clear the argument lists.
    pub fn clear(&mut self) {
        self.pos.clear();
        self.key.clear();
    }

    /// Whether both the positional and keyword argument lists are empty.
    pub fn is_empty(&self) -> bool {
        self.pos.is_empty() && self.key.is_empty()
    }
}

fn expect<E: ExpressionKind>(opt: ParseResult<Option<Spanned<E>>>) -> ParseResult<Spanned<E>> {
    match opt {
        Ok(Some(spanned)) => Ok(spanned),
        Ok(None) => error!("expected {}", E::NAME),
        Err(e) => Err(e),
    }
}

/// A positional argument passed to a function.
pub type PosArg = Expression;

/// A keyword argument passed to a function.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyArg {
    pub key: Spanned<Ident>,
    pub value: Spanned<Expression>,
}

/// Either a positional or keyword argument.
#[derive(Debug, Clone, PartialEq)]
pub enum DynArg {
    Pos(Spanned<PosArg>),
    Key(Spanned<KeyArg>),
}

/// An argument or return value.
#[derive(Clone, PartialEq)]
pub enum Expression {
    Ident(Ident),
    Str(String),
    Num(f64),
    Size(Size),
    Bool(bool),
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use Expression::*;
        match self {
            Ident(i) => write!(f, "{}", i),
            Str(s) => write!(f, "{:?}", s),
            Num(n) => write!(f, "{}", n),
            Size(s) => write!(f, "{}", s),
            Bool(b) => write!(f, "{}", b),
        }
    }
}

debug_display!(Expression);

/// An identifier.
#[derive(Clone, PartialEq)]
pub struct Ident(pub String);

impl Ident {
    fn new(string: String) -> ParseResult<Ident> {
        if is_identifier(&string) {
            Ok(Ident(string))
        } else {
            error!("invalid identifier: `{}`", string);
        }
    }
}

impl Display for Ident {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

debug_display!(Ident);

/// Whether this word is a valid unicode identifier.
fn is_identifier(string: &str) -> bool {
    let mut chars = string.chars();

    match chars.next() {
        Some('-') => (),
        Some(c) if UnicodeXID::is_xid_start(c) => (),
        _ => return false,
    }

    while let Some(c) = chars.next() {
        match c {
            '.' | '-' => (),
            c if UnicodeXID::is_xid_continue(c) => (),
            _ => return false,
        }
    }

    true
}

/// Kinds of expressions.
pub trait ExpressionKind: Sized {
    const NAME: &'static str;

    /// Create from expression.
    fn from_expr(expr: Spanned<Expression>) -> ParseResult<Self>;
}

macro_rules! kind {
    ($type:ty, $name:expr, $($patterns:tt)*) => {
        impl ExpressionKind for $type {
            const NAME: &'static str = $name;

            fn from_expr(expr: Spanned<Expression>) -> ParseResult<Self> {
                #[allow(unreachable_patterns)]
                Ok(match expr.v {
                    $($patterns)*,
                    _ => error!("expected {}", Self::NAME),
                })
            }
        }
    };
}

kind!(Expression, "expression", e                         => e);
kind!(Ident,      "identifier", Expression::Ident(ident)  => ident);
kind!(String,     "string",     Expression::Str(string)   => string);
kind!(f64,        "number",     Expression::Num(num)      => num);
kind!(bool,       "boolean",    Expression::Bool(boolean) => boolean);
kind!(Size,       "size",       Expression::Size(size)    => size);
kind!(ScaleSize,  "number or size",
    Expression::Size(size) => ScaleSize::Absolute(size),
    Expression::Num(scale) => ScaleSize::Scaled(scale as f32)
);
