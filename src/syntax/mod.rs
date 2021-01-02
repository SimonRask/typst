//! Syntax types.

mod expr;
mod ident;
mod node;
mod span;
mod token;

pub use expr::*;
pub use ident::*;
pub use node::*;
pub use span::*;
pub use token::*;

/// A collection of nodes which form a tree together with their children.
pub type Tree = SpanVec<Node>;
