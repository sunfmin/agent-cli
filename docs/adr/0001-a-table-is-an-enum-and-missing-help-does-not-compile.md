# A table is an enum, and an entry without help does not compile

A table's entries are its enum's variants, read by `#[derive(Help)]` from the
enum itself, not collected at link time (`inventory`, `linkme`) or listed by
hand: a code added to the enum is in the manual, and one with no doc comment
or `#[help]` is a compile error naming it, never a silent gap as with ruff's
`OptionsMetadata`, which skips undocumented fields. The derive is named `Help`,
the word an author reaches for, but implements the trait `agent_help::Table`:
the name `Help` belongs to the manual a tool assembles from its tables, topics
and commands.
