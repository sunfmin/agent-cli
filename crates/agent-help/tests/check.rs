//! `agent_help::check`, the one test that keeps a manual whole, and
//! `agent_help::code_blocks`, which hands a tool the examples its manual
//! shows.

use agent_help::{CodeBlock, Fault, Help, Topic};
use clap::Command;

agent_help::binary!("plans");

/// Run `plans login` once. Plans are kept as [retention](help:retention) says.
#[derive(Topic)]
#[topic(name = "sign-in")]
struct SignIn;

/// A plan is kept {const:RETENTION}.
///
/// ```sh
/// plans plan list --older-than {const:RETENTION_DAYS}d
/// ```
#[derive(Topic)]
#[topic(name = "retention")]
struct Retention;

/// Another topic named `sign-in`.
#[derive(Topic)]
#[topic(name = "sign-in")]
struct SignInAgain;

/// Why `plans` refused a request.
#[derive(agent_help::Help)]
#[allow(dead_code)]
enum Refusal {
    /// The name is taken by another plan.
    NameTaken,
    /// The plan is over its limit. Raise it:
    ///
    /// ```sh
    /// plans plan raise --by 10
    /// ```
    OverLimit,
}

fn tool() -> Command {
    Command::new("plans")
        .about("Keeps plans; see [signing in](help:sign-in).")
        .subcommand(
            Command::new("plan")
                .next_help_heading("Run by the agent")
                .about("Works with plans.")
                .subcommand(Command::new("create").long_about(
                    "Creates a plan.\n\n\
                     ```rust\nlet plan = plans::create(\"q3\");\n```\n\n\
                     ```sh\nplans plan create q3\n```\n\n\
                     It may refuse:\n\n{table:refusal}",
                )),
        )
        .subcommand(
            Command::new("debug")
                .hide(true)
                .about("Dumps state.\n\n```rust,no_run\nplans::dump();\n```"),
        )
}

fn help() -> Help {
    Help::new(tool())
        .topic::<SignIn>()
        .topic::<Retention>()
        .table::<Refusal>("refusal")
        .constant("RETENTION", "for 30 days")
        .constant("RETENTION_DAYS", 30)
}

fn faults(error: agent_help::Error) -> Vec<String> {
    error.faults().iter().map(Fault::to_string).collect()
}

#[test]
fn a_whole_manual_passes() {
    assert_eq!(agent_help::check(&help()), Ok(()));
}

#[test]
fn a_command_without_text_is_a_fault_hidden_or_not() {
    let help = Help::new(
        Command::new("plans")
            .subcommand(Command::new("serve").next_help_heading("Run by the operator"))
            .subcommand(Command::new("debug").hide(true).about("  ")),
    );
    assert_eq!(
        faults(agent_help::check(&help).unwrap_err()),
        [
            "`plans` has no text: write its doc comment, and keep it as written \
             with `#[command(verbatim_doc_comment)]`",
            "`plans serve` has no text: write its doc comment, and keep it as written \
             with `#[command(verbatim_doc_comment)]`",
            "`plans debug` has no text: write its doc comment, and keep it as written \
             with `#[command(verbatim_doc_comment)]`",
        ]
    );
}

#[test]
fn a_link_naming_no_topic_is_a_fault_naming_where_it_is() {
    let help = Help::new(
        Command::new("plans")
            .about("See [billing](help:billing), then [signing in](help:sign-in).")
            .subcommand(
                Command::new("debug")
                    .hide(true)
                    .about("See [nothing](help:)."),
            ),
    )
    .topic::<SignIn>();
    assert_eq!(
        faults(agent_help::check(&help).unwrap_err()),
        [
            "`help:billing` in `plans` names no topic; the topics are: `sign-in`",
            "`help:retention` in topic `sign-in` names no topic; the topics are: `sign-in`",
            "`help:` in `plans debug` names no topic; the topics are: `sign-in`",
        ]
    );
}

#[test]
fn a_link_in_a_table_is_checked_where_it_is_written() {
    /// Codes.
    #[derive(agent_help::Help)]
    #[allow(dead_code)]
    enum Codes {
        /// See [signing in](help:sign-in).
        NotSignedIn,
    }
    let help = Help::new(Command::new("plans").about("{table:codes}")).table::<Codes>("codes");
    assert_eq!(
        faults(agent_help::check(&help).unwrap_err()),
        ["`help:sign-in` in table `codes` names no topic; the topics are: none"]
    );
}

#[test]
fn text_that_only_looks_like_a_link_is_kept() {
    let help = Help::new(
        Command::new("plans")
            .about("[a](help: two words) and [b](https://example.com/help:x) and (help:x)"),
    );
    assert_eq!(agent_help::check(&help), Ok(()));
}

#[test]
fn two_topics_sharing_a_name_is_a_fault() {
    let help = help().topic::<SignInAgain>();
    let duplicate = "topic `sign-in` is added twice: give each topic a name of its own";
    assert_eq!(faults(agent_help::check(&help).unwrap_err()), [duplicate]);
    assert_eq!(faults(help.manual().unwrap_err()), [duplicate]);
}

#[test]
fn an_unresolved_placeholder_is_a_fault() {
    let help = Help::new(
        Command::new("plans")
            .about("Keeps plans.")
            .subcommand(Command::new("debug").hide(true).about("{const:MAX}")),
    );
    assert_eq!(
        faults(agent_help::check(&help).unwrap_err()),
        ["`{const:MAX}` in `plans debug` names no constant; the constants are: none"]
    );
}

#[test]
fn every_fault_is_named_at_once() {
    let help = Help::new(
        Command::new("plans")
            .about("See [billing](help:billing).")
            .subcommand(Command::new("serve").about("{table:codes}")),
    )
    .topic::<Retention>()
    .topic::<Retention>()
    .constant("RETENTION", "for 30 days")
    .constant("RETENTION_DAYS", 30);
    assert_eq!(
        faults(agent_help::check(&help).unwrap_err()),
        [
            "`help:billing` in `plans` names no topic; the topics are: `retention`, `retention`",
            "`plans serve` has no group: declare who runs it where it is defined, \
             with `#[command(next_help_heading = \"...\")]`",
            "topic `retention` is added twice: give each topic a name of its own",
            "`{table:codes}` in `plans serve` names no table; the tables are: none",
        ]
    );
}

#[test]
fn code_blocks_hands_over_every_block_of_a_language_with_where_it_is() {
    assert_eq!(
        agent_help::code_blocks(&help(), "rust"),
        [
            CodeBlock {
                at: "`plans plan create`".to_owned(),
                code: "let plan = plans::create(\"q3\");\n".to_owned(),
            },
            CodeBlock {
                at: "`plans debug`".to_owned(),
                code: "plans::dump();\n".to_owned(),
            },
        ]
    );
}

#[test]
fn code_blocks_are_as_the_manual_shows_them() {
    assert_eq!(
        agent_help::code_blocks(&help(), "sh"),
        [
            CodeBlock {
                at: "`plans plan create`".to_owned(),
                code: "plans plan create q3\n".to_owned(),
            },
            CodeBlock {
                at: "`plans plan create`".to_owned(),
                code: "plans plan raise --by 10\n".to_owned(),
            },
            CodeBlock {
                at: "topic `retention`".to_owned(),
                code: "plans plan list --older-than 30d\n".to_owned(),
            },
        ]
    );
    assert_eq!(
        agent_help::code_blocks(&help(), "go"),
        Vec::<CodeBlock>::new()
    );
}
