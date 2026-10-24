use agent_help::{Entry, Help, Table};

/// Why a request was refused.
#[derive(Help)]
#[allow(dead_code)]
enum Refusal {
    /// The name is taken.
    NameTaken,
    /// The plan is over its limit.
    ///
    /// Raise it with:
    ///
    /// ```sh
    /// tool plan raise --by 10
    /// ```
    ///
    /// - one
    ///   - nested
    OverLimit { by: u32 },
    /// What a developer reads.
    #[help = "What the user reads."]
    HTTPTimeout(u64),
}

#[test]
fn every_variant_is_an_entry_in_declaration_order_named_in_snake_case() {
    let names: Vec<&str> = Refusal::ENTRIES.iter().map(|e| e.name).collect();
    assert_eq!(names, ["name_taken", "over_limit", "http_timeout"]);
}

#[test]
fn a_doc_comment_is_kept_verbatim_with_one_leading_space_dropped() {
    assert_eq!(Refusal::ENTRIES[0].markdown, "The name is taken.");
    assert_eq!(
        Refusal::ENTRIES[1].markdown,
        "The plan is over its limit.\n\nRaise it with:\n\n```sh\ntool plan raise --by 10\n```\n\n- one\n  - nested"
    );
}

#[test]
fn help_attribute_wins_over_the_doc_comment() {
    assert_eq!(Refusal::ENTRIES[2].markdown, "What the user reads.");
}

#[test]
fn a_value_knows_its_entry() {
    assert_eq!(
        Refusal::OverLimit { by: 3 }.entry(),
        &Entry {
            name: "over_limit",
            markdown: Refusal::ENTRIES[1].markdown,
        }
    );
    assert_eq!(Refusal::HTTPTimeout(5).entry().name, "http_timeout");
}

#[test]
fn the_table_renders_as_one_markdown_list_in_declaration_order() {
    assert_eq!(
        Refusal::markdown(),
        "\
- `name_taken`: The name is taken.
- `over_limit`: The plan is over its limit.

  Raise it with:

  ```sh
  tool plan raise --by 10
  ```

  - one
    - nested
- `http_timeout`: What the user reads.
"
    );
}

/// Help from a file, as a long text is kept.
#[derive(Help)]
#[allow(dead_code)]
enum FromFile {
    #[help = include_str!("help/from_file.md")]
    Included,
}

#[test]
fn help_attribute_takes_an_included_file() {
    assert_eq!(FromFile::ENTRIES[0].markdown, "Read from a file.\n");
}

#[test]
fn compile_errors() {
    trybuild::TestCases::new().compile_fail("tests/compile_fail/*.rs");
}

/// A table generic over what a variant holds.
#[derive(Help)]
#[allow(dead_code)]
enum Generic<T: Copy> {
    /// Holds a value.
    Holds(T),
}

/// A table with no entries yet.
#[derive(Help)]
enum Empty {}

#[test]
fn a_generic_or_empty_enum_is_a_table() {
    assert_eq!(Generic::Holds(1u8).entry().name, "holds");
    assert!(Empty::ENTRIES.is_empty());
    assert_eq!(Empty::markdown(), "");
}
