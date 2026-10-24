//! A command-line tool's help, written beside what it describes and
//! assembled into one Markdown manual whose reader is an agent.
//!
//! # Tables
//!
//! A table is an enum whose variants are a closed set of facts, such as a
//! tool's refusal codes. `#[derive(Help)]` makes each variant's doc comment
//! its help, so the enum that is the codes' one home is also their manual:
//!
//! ````
//! use agent_help::{Help, Table};
//!
//! /// Why `tool` refused a request.
//! #[derive(Help)]
//! enum Refusal {
//!     /// The name is taken by another plan.
//!     NameTaken,
//!     /// The plan is over its limit.
//!     ///
//!     /// Raise the limit, then run the command again:
//!     ///
//!     /// ```sh
//!     /// tool plan raise --by 10
//!     /// ```
//!     OverLimit { by: u32 },
//!     /// Raised when `client::DEADLINE` passes before the server answers.
//!     #[help = "The server did not answer in time: run the command again."]
//!     Timeout,
//! }
//!
//! assert_eq!(Refusal::Timeout.entry().name, "timeout");
//! assert_eq!(
//!     Refusal::markdown(),
//!     "\
//! - `name_taken`: The name is taken by another plan.
//! - `over_limit`: The plan is over its limit.
//!
//!   Raise the limit, then run the command again:
//!
//!   ```sh
//!   tool plan raise --by 10
//!   ```
//! - `timeout`: The server did not answer in time: run the command again.
//! "
//! );
//!
//! // Only the codes one command can meet, as the same list, in this order.
//! assert_eq!(
//!     Refusal::markdown_of(&[Refusal::Timeout, Refusal::NameTaken]),
//!     "\
//! - `timeout`: The server did not answer in time: run the command again.
//! - `name_taken`: The name is taken by another plan.
//! "
//! );
//! ````
//!
//! - A variant's name is its identifier in snake_case, as a tool prints the
//!   code.
//! - Its help is its doc comment as written, one leading space of each `/// `
//!   line dropped: Markdown, never wrapped or rendered. Write each paragraph
//!   on one line, and start with a sentence: in the list it follows the name.
//! - `#[help = "..."]` replaces the doc comment, for when what the user reads
//!   differs from what a developer does; `#[help = include_str!("x.md")]`
//!   takes a file.
//! - A variant with neither, or with only empty `///` lines, does not
//!   compile, and the error names every such variant.
//! - [`Table::markdown`] lists every entry; [`Table::markdown_of`] lists the
//!   values given, in their order, each item exactly as `markdown` writes it:
//!   for a page that lists part of a table, such as one command's refusals.
//!
//! # Topics
//!
//! A topic is a piece of the manual that belongs to no single command; see
//! [`Topic`].
//!
//! # The manual
//!
//! [`struct@Help`] assembles a tool's manual from clap's command tree, its
//! topics and its tables, and answers `--help`, `<command> --help` and
//! `help <topic>` from it. `examples/plans.rs` is a tool wired to one.

mod manual;
mod topic;

pub use agent_help_derive::Help;
pub use manual::{Error, Fault, Help};
pub use topic::{Topic, TopicRef};

/// One variant of a [`Table`]: its name and its help.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
    /// The variant's identifier in snake_case, e.g. `over_limit`.
    pub name: &'static str,
    /// The variant's help, Markdown as written.
    pub markdown: &'static str,
}

/// An enum whose variants carry their help; implemented by `#[derive(Help)]`.
pub trait Table {
    /// Every variant's entry, in declaration order.
    const ENTRIES: &'static [Entry];

    /// This value's variant's entry.
    fn entry(&self) -> &'static Entry;

    /// The table as one Markdown list, one item per variant in declaration
    /// order: the name in backticks, then its help, whose later lines are
    /// indented to stay inside the item. Ends with a newline.
    fn markdown() -> String {
        list(Self::ENTRIES)
    }

    /// The given values' entries as the list [`markdown`](Table::markdown)
    /// gives for all of them, one item per value in the order given; empty
    /// when no value is.
    fn markdown_of<'a>(values: impl IntoIterator<Item = &'a Self>) -> String
    where
        Self: 'a,
    {
        list(values.into_iter().map(Self::entry))
    }
}

/// Entries as a Markdown list, each item ended by a newline.
fn list<'a>(entries: impl IntoIterator<Item = &'a Entry>) -> String {
    let mut list = String::new();
    for entry in entries {
        list.push_str("- `");
        list.push_str(entry.name);
        list.push_str("`: ");
        for (i, line) in entry.markdown.lines().enumerate() {
            if i > 0 {
                list.push('\n');
                if !line.is_empty() {
                    list.push_str("  ");
                }
            }
            list.push_str(line);
        }
        list.push('\n');
    }
    list
}
