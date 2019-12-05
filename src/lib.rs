//! The compiler for the _Typst_ typesetting language.
//!
//! # Steps
//! - **Parsing:** The parsing step first transforms a plain string into an
//!   [iterator of tokens](crate::syntax::Tokens). Then, a parser constructs a
//!   syntax tree from the token stream. The structures describing the tree can
//!   be found in the [syntax](crate::syntax) module.
//! - **Layouting:** The next step is to transform the syntax tree into a
//!   portable representation of the typesetted document. Types for these can be
//!   found in the [layout] module. A finished layout reading for exporting is a
//!   [multi layout](crate::layout::MultiLayout) consisting of multiple boxes (or
//!   pages).
//! - **Exporting:** The finished document can finally be exported into a supported
//!   format. Submodules for these formats are located in the [export](crate::export)
//!   module. Currently, the only supported output format is _PDF_.

pub extern crate toddle;

use std::cell::RefCell;
use smallvec::smallvec;

use toddle::query::{FontLoader, FontProvider, SharedFontLoader};
use toddle::Error as FontError;

use crate::func::Scope;
use crate::layout::{layout_tree, MultiLayout, LayoutContext};
use crate::layout::{LayoutAxes, LayoutAlignment, Axis, Alignment};
use crate::layout::{LayoutResult, LayoutSpace};
use crate::syntax::{parse, SyntaxTree, ParseContext, Span, ParseResult};
use crate::style::{LayoutStyle, PageStyle, TextStyle};

#[macro_use]
mod macros;
pub mod export;
#[macro_use]
pub mod func;
pub mod layout;
pub mod library;
pub mod size;
pub mod style;
pub mod syntax;

/// Transforms source code into typesetted layouts.
///
/// A typesetter can be configured through various methods.
pub struct Typesetter<'p> {
    /// The font loader shared by all typesetting processes.
    loader: SharedFontLoader<'p>,
    /// The base layouting style.
    style: LayoutStyle,
}

impl<'p> Typesetter<'p> {
    /// Create a new typesetter.
    #[inline]
    pub fn new() -> Typesetter<'p> {
        Typesetter {
            loader: RefCell::new(FontLoader::new()),
            style: LayoutStyle::default(),
        }
    }

    /// Set the base page style.
    #[inline]
    pub fn set_page_style(&mut self, style: PageStyle) {
        self.style.page = style;
    }

    /// Set the base text style.
    #[inline]
    pub fn set_text_style(&mut self, style: TextStyle) {
        self.style.text = style;
    }

    /// Add a font provider to the context of this typesetter.
    #[inline]
    pub fn add_font_provider<P: 'p>(&mut self, provider: P)
    where P: FontProvider {
        self.loader.get_mut().add_provider(provider);
    }

    /// A reference to the backing font loader.
    #[inline]
    pub fn loader(&self) -> &SharedFontLoader<'p> {
        &self.loader
    }

    /// Parse source code into a syntax tree.
    pub fn parse(&self, src: &str) -> ParseResult<SyntaxTree> {
        let scope = Scope::with_std();
        parse(src, ParseContext { scope: &scope })
    }

    /// Layout a syntax tree and return the produced layout.
    pub fn layout(&self, tree: &SyntaxTree) -> LayoutResult<MultiLayout> {
        Ok(layout_tree(
            &tree,
            LayoutContext {
                loader: &self.loader,
                top_level: true,
                style: &self.style,
                spaces: smallvec![LayoutSpace {
                    dimensions: self.style.page.dimensions,
                    expand: (true, true),
                    padding: self.style.page.margins,
                }],
                axes: LayoutAxes::new(Axis::LeftToRight, Axis::TopToBottom),
                alignment: LayoutAlignment::new(Alignment::Origin, Alignment::Origin),
            },
        )?)
    }

    /// Process source code directly into a layout.
    pub fn typeset(&self, src: &str) -> Result<MultiLayout, TypesetError> {
        let tree = self.parse(src)?;
        let layout = self.layout(&tree)?;
        Ok(layout)
    }
}

/// The result type for typesetting.
pub type TypesetResult<T> = Result<T, TypesetError>;

/// The error type for typesetting.
pub struct TypesetError {
    message: String,
    span: Option<Span>,
}

impl TypesetError {
    /// Create a new typesetting error.
    pub fn with_message(message: String) -> TypesetError {
        TypesetError { message, span: None }
    }
}

error_type! {
    self: TypesetError,
    show: f => {
        write!(f, "{}", self.message)?;
        if let Some(span) = self.span {
            write!(f, " at {}", span)?;
        }
        Ok(())
    },
    from: (err: std::io::Error, TypesetError::with_message(err.to_string())),
    from: (err: FontError, TypesetError::with_message(err.to_string())),
}
