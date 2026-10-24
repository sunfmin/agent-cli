//! `#[derive(Topic)]`: an empty struct, its `#[topic(name = "...")]` and its
//! doc comments made into an `agent_help::Topic`.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, LitStr, Result};

use crate::docs;

pub fn expand(input: &DeriveInput) -> Result<TokenStream> {
    let ty = &input.ident;
    match &input.data {
        Data::Struct(data) if data.fields.is_empty() => {}
        _ => {
            return Err(Error::new_spanned(
                ty,
                format!("a topic is an empty struct: write `struct {ty};`"),
            ));
        }
    }
    let name = name(input)?;
    let markdown = docs::markdown(&input.attrs).ok_or_else(|| {
        Error::new_spanned(
            ty,
            format!(
                "topic `{ty}` has no doc comment, and its doc comment is its Markdown: \
                 write it as `///` lines, or as `#[doc = include_str!(\"{}.md\")]`",
                name.value()
            ),
        )
    })?;
    Ok(quote! {
        impl ::agent_help::Topic for #ty {
            const NAME: &'static str = #name;
            const MARKDOWN: &'static str = #markdown;
            const REFERENCE: ::agent_help::TopicRef =
                ::agent_help::TopicRef::__new(crate::__AGENT_HELP_BINARY, Self::NAME);
        }
    })
}

/// The `name` of the one `#[topic(...)]`, checked to be kebab-case.
fn name(input: &DeriveInput) -> Result<LitStr> {
    let mut name: Option<LitStr> = None;
    for attr in input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("topic"))
    {
        attr.parse_nested_meta(|meta| {
            if !meta.path.is_ident("name") {
                return Err(meta.error("a topic takes only `name = \"...\"`"));
            }
            if name.is_some() {
                return Err(meta.error("a topic has one name"));
            }
            let value: LitStr = meta.value()?.parse()?;
            if !is_kebab_case(&value.value()) {
                return Err(Error::new_spanned(
                    &value,
                    "a topic's name is kebab-case: lowercase words of letters and digits \
                     joined by `-`, such as `sign-in`",
                ));
            }
            name = Some(value);
            Ok(())
        })?;
    }
    let ty = &input.ident;
    name.ok_or_else(|| {
        Error::new_spanned(
            ty,
            format!(
                "topic `{ty}` has no name: add `#[topic(name = \"...\")]`, \
                 the name `<binary> help <name>` answers to"
            ),
        )
    })
}

fn is_kebab_case(name: &str) -> bool {
    name.split('-').all(|word| {
        !word.is_empty()
            && word
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    fn error_of(input: DeriveInput) -> String {
        expand(&input)
            .expect_err("the derive accepted it")
            .to_string()
    }

    #[test]
    fn a_topic_without_a_doc_comment_is_refused_naming_it() {
        let error = error_of(parse_quote! {
            #[topic(name = "sign-in")]
            struct SignIn;
        });
        assert_eq!(
            error,
            "topic `SignIn` has no doc comment, and its doc comment is its Markdown: \
             write it as `///` lines, or as `#[doc = include_str!(\"sign-in.md\")]`"
        );
    }

    #[test]
    fn a_blank_doc_comment_is_no_doc_comment() {
        let error = error_of(parse_quote! {
            ///
            #[doc = "  "]
            #[topic(name = "sign-in")]
            struct SignIn;
        });
        assert!(
            error.starts_with("topic `SignIn` has no doc comment"),
            "{error}"
        );
    }

    #[test]
    fn a_topic_without_a_name_is_refused() {
        let error = error_of(parse_quote! {
            /// How to sign in.
            struct SignIn;
        });
        assert!(error.starts_with("topic `SignIn` has no name"), "{error}");
    }

    #[test]
    fn a_name_that_is_not_kebab_case_is_refused() {
        for name in [
            "SignIn", "sign_in", "sign--in", "-sign", "sign-", "", "sign in",
        ] {
            let error = error_of(parse_quote! {
                /// How to sign in.
                #[topic(name = #name)]
                struct SignIn;
            });
            assert!(
                error.starts_with("a topic's name is kebab-case"),
                "{name}: {error}"
            );
        }
    }

    #[test]
    fn a_kebab_case_name_is_accepted() {
        for name in ["sign-in", "s3", "object-store-v2"] {
            assert!(is_kebab_case(name), "{name}");
        }
    }

    #[test]
    fn a_second_name_is_refused() {
        let error = error_of(parse_quote! {
            /// How to sign in.
            #[topic(name = "sign-in")]
            #[topic(name = "login")]
            struct SignIn;
        });
        assert_eq!(error, "a topic has one name");
    }

    #[test]
    fn an_unknown_key_is_refused() {
        let error = error_of(parse_quote! {
            /// How to sign in.
            #[topic(title = "Sign in")]
            struct SignIn;
        });
        assert_eq!(error, "a topic takes only `name = \"...\"`");
    }

    #[test]
    fn a_topic_with_fields_or_an_enum_is_refused() {
        for input in [
            parse_quote! {
                /// How to sign in.
                #[topic(name = "sign-in")]
                struct SignIn { token: String }
            },
            parse_quote! {
                /// How to sign in.
                #[topic(name = "sign-in")]
                enum SignIn {}
            },
        ] {
            assert_eq!(
                error_of(input),
                "a topic is an empty struct: write `struct SignIn;`"
            );
        }
    }
}
