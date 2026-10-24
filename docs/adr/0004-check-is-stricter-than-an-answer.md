# `check` is stricter than an answer: a command without text fails the test, not the tool

`agent_help::check(&help)` is the one test a tool writes for its manual. It finds every fault an answer finds — `Help::clap()` refuses a manual with an ungrouped command, a placeholder or a topic link naming nothing, or two topics of one name — and one more: a command, the tool's own or a subcommand, hidden or not, with no text.

An answer (`manual()`, `command_help()`, `clap()`) does not refuse a command without text. A missing text leaves the manual incomplete but not wrong: what it shows is still true, so a tool keeps starting. The test is where completeness is enforced, so it fails in the tool's own `cargo test`, before release, rather than at a user's `--help`.

## Considered Options

- **Every answer refuses a command without text**: a tool that ships a stub subcommand stops starting at all, for a gap that misleads nobody.
- **`check` only finds what answers find, plus nothing**: then nothing ever requires the text, and a gap stays silent — the failure ruff's `OptionsMetadata` has for fields without `#[option]`.

## Consequences

A tool's manual is whole only if it runs `check` in a test; `examples/plans.rs` does, as `the_manual_is_whole`.
