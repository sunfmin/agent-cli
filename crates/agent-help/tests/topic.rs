//! A topic as a tool writes one: a type deriving `Topic`, its Markdown in a
//! file of its own, and a message pointing at it by its reference.

use agent_help::Topic;

agent_help::binary!("bk-server");

/// How to sign in.
#[derive(Topic)]
#[topic(name = "sign-in")]
#[doc = include_str!("sign-in.md")]
struct SignIn;

/// Keeping files on an object store.
///
/// ```sh
/// bk-server store set s3://bucket
/// ```
///
///   - indented two more spaces
#[derive(Topic)]
#[topic(name = "object-store")]
struct ObjectStore;

fn refusal(reason: &str) -> String {
    format!("{reason} (run `{}`)", SignIn::REFERENCE)
}

#[test]
fn a_topic_has_its_name() {
    assert_eq!(SignIn::NAME, "sign-in");
    assert_eq!(ObjectStore::NAME, "object-store");
}

#[test]
fn a_topic_from_a_markdown_file_is_that_file_verbatim() {
    assert_eq!(
        SignIn::MARKDOWN,
        concat!("How to sign in.\n", include_str!("sign-in.md"))
    );
}

#[test]
fn doc_comment_lines_are_kept_verbatim_less_one_leading_space() {
    assert_eq!(
        ObjectStore::MARKDOWN,
        "Keeping files on an object store.\n\
         \n\
         ```sh\n\
         bk-server store set s3://bucket\n\
         ```\n\
         \n  - indented two more spaces"
    );
}

#[test]
fn the_reference_names_the_binary_once_for_the_tool() {
    assert_eq!(SignIn::REFERENCE.to_string(), "bk-server help sign-in");
    assert_eq!(
        ObjectStore::REFERENCE.to_string(),
        "bk-server help object-store"
    );
    assert_eq!(SignIn::REFERENCE.topic(), "sign-in");
}

#[test]
fn an_error_message_carries_the_reference() {
    assert_eq!(
        refusal("not signed in"),
        "not signed in (run `bk-server help sign-in`)"
    );
}

#[test]
fn the_reference_is_a_constant() {
    const FIX: agent_help::TopicRef = SignIn::REFERENCE;
    assert_eq!(FIX, SignIn::REFERENCE);
}
