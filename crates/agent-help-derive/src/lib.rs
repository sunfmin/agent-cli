//! The derive macros of agent-help. Their documentation, and the traits they
//! implement, are agent-help's: depend on it, not on this crate.

use heck::ToSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Meta, Variant, parse_macro_input};

mod docs;
mod topic;

/// Implements `agent_help::Topic`; see there.
#[proc_macro_derive(Topic, attributes(topic))]
pub fn derive_topic(input: TokenStream) -> TokenStream {
    topic::expand(&parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Implements `agent_help::Table` for an enum: each variant's help is its
/// `#[help = ...]`, else its doc comment; a variant with neither is an error.
#[proc_macro_derive(Help, attributes(help))]
pub fn derive_help(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    table(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn table(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let Data::Enum(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "#[derive(Help)] is for an enum: its variants are the table's entries",
        ));
    };

    // Every variant is read before any error is returned, so one build names
    // every variant that lacks help.
    let mut entries = Vec::new();
    let mut errors: Option<syn::Error> = None;
    for variant in &data.variants {
        match help(variant) {
            Ok(markdown) => entries.push((variant, markdown)),
            Err(e) => match &mut errors {
                Some(all) => all.combine(e),
                None => errors = Some(e),
            },
        }
    }
    if let Some(errors) = errors {
        return Err(errors);
    }

    let enum_name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let consts = entries.iter().map(|(variant, markdown)| {
        let name = variant.ident.to_string().to_snake_case();
        quote!(::agent_help::Entry { name: #name, markdown: #markdown })
    });
    let arms = entries.iter().enumerate().map(|(i, (variant, _))| {
        let ident = &variant.ident;
        quote!(Self::#ident { .. } => &<Self as ::agent_help::Table>::ENTRIES[#i])
    });
    Ok(quote! {
        impl #impl_generics ::agent_help::Table for #enum_name #type_generics #where_clause {
            const ENTRIES: &'static [::agent_help::Entry] = &[#(#consts),*];

            fn entry(&self) -> &'static ::agent_help::Entry {
                match *self {
                    #(#arms,)*
                }
            }
        }
    })
}

/// A variant's help, as an expression of type `&'static str`.
fn help(variant: &Variant) -> syn::Result<TokenStream2> {
    let mut overrides = variant.attrs.iter().filter(|a| a.path().is_ident("help"));
    if let Some(attr) = overrides.next() {
        if let Some(second) = overrides.next() {
            return Err(syn::Error::new_spanned(
                second,
                format!("variant `{}` has two #[help]: keep one", variant.ident),
            ));
        }
        let Meta::NameValue(pair) = &attr.meta else {
            return Err(syn::Error::new_spanned(
                attr,
                "write #[help = \"...\"] or #[help = include_str!(\"...\")]",
            ));
        };
        let value = &pair.value;
        return Ok(quote!(#value));
    }

    docs::markdown(&variant.attrs).ok_or_else(|| {
        syn::Error::new_spanned(
            &variant.ident,
            format!(
                "variant `{}` has no help: write a doc comment (`///`) on it, \
                 or #[help = \"...\"]",
                variant.ident
            ),
        )
    })
}
