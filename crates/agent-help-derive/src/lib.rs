//! The derive macros of agent-help. Their documentation, and the traits they
//! implement, are agent-help's: depend on it, not on this crate.

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod docs;
mod topic;

/// Implements `agent_help::Topic`; see there.
#[proc_macro_derive(Topic, attributes(topic))]
pub fn derive_topic(input: TokenStream) -> TokenStream {
    topic::expand(&parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
