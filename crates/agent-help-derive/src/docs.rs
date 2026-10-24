//! An item's doc comments, read as the Markdown an author wrote.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Expr, ExprLit, Lit, Meta};

/// The Markdown of `attrs`' doc comments, as one `&'static str` expression:
/// each `#[doc]` in order, joined by a newline. A `///` line loses the one
/// space after its slashes and is otherwise kept as written; a `#[doc = ...]`
/// whose value is a macro (`include_str!("x.md")`) is that value, verbatim.
///
/// `None` when there is no doc comment, or only blank ones.
pub fn markdown(attrs: &[Attribute]) -> Option<TokenStream> {
    let parts: Vec<Part> = attrs.iter().filter_map(Part::of).collect();
    if parts.iter().all(Part::is_blank) {
        return None;
    }
    let mut lines = TokenStream::new();
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            lines.extend(quote! { "\n", });
        }
        lines.extend(part.to_tokens());
        lines.extend(quote! { , });
    }
    Some(quote! { ::core::concat!(#lines) })
}

/// One `#[doc]`.
enum Part {
    /// Written in the source: `///` or `#[doc = "..."]`.
    Text(String),
    /// Anything else, `include_str!` above all: known only once expanded.
    Expr(Expr),
}

impl Part {
    fn of(attr: &Attribute) -> Option<Self> {
        let Meta::NameValue(doc) = &attr.meta else {
            return None;
        };
        if !doc.path.is_ident("doc") {
            return None;
        }
        Some(match &doc.value {
            Expr::Lit(ExprLit {
                lit: Lit::Str(text),
                ..
            }) => Part::Text(without_comment_space(&text.value())),
            value => Part::Expr(value.clone()),
        })
    }

    fn is_blank(&self) -> bool {
        matches!(self, Part::Text(text) if text.trim().is_empty())
    }

    fn to_tokens(&self) -> TokenStream {
        match self {
            Part::Text(text) => quote! { #text },
            Part::Expr(value) => quote! { #value },
        }
    }
}

/// `text` less the one space `/// ` puts at the start of each of its lines.
fn without_comment_space(text: &str) -> String {
    text.split('\n')
        .map(|line| line.strip_prefix(' ').unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n")
}
