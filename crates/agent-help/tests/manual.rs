//! A tool's manual assembled by `Help`: from clap's command tree, its topics
//! and its tables, with every placeholder replaced.

use agent_help::{Fault, Help, Topic};
use clap::Command;
use clap::error::ErrorKind;

agent_help::binary!("plans");

/// Run `plans login` once; it keeps the token in the keychain.
#[derive(Topic)]
#[topic(name = "sign-in")]
struct SignIn;

/// A plan is kept {const:RETENTION}.
#[derive(Topic)]
#[topic(name = "retention")]
struct Retention;

/// Why `plans` refused a request.
#[derive(agent_help::Help)]
#[allow(dead_code)]
enum Refusal {
    /// The name is taken by another plan.
    NameTaken,
    /// There are {const:MAX_PLANS} plans already.
    OverLimit,
}

/// A topic of another tool: a crate names one binary, so this one is written
/// out as another crate's derive would write it.
struct ForeignSignIn;

impl Topic for ForeignSignIn {
    const NAME: &'static str = "sign-in";
    const MARKDOWN: &'static str = "How to sign in to bk-server.";
    const REFERENCE: agent_help::TopicRef = agent_help::TopicRef::__new("bk-server", Self::NAME);
}

fn tool() -> Command {
    Command::new("plans")
        .disable_help_subcommand(true)
        .about("Keeps plans.")
        .long_about("Keeps plans, at most {const:MAX_PLANS}.\n\nRun `plans plan list` first.")
        .subcommand(
            Command::new("plan")
                .next_help_heading("Run by the agent")
                .about("Works with plans.")
                .subcommand(
                    Command::new("create")
                        .about("Creates a plan.")
                        .long_about("Creates a plan.\n\nIt may refuse:\n\n{table:refusal}"),
                )
                .subcommand(Command::new("list").about("Lists every plan.")),
        )
        .subcommand(
            Command::new("serve")
                .next_help_heading("Run by the operator")
                .about("Serves the plans."),
        )
        .subcommand(
            Command::new("help")
                .next_help_heading("Run by the agent")
                .about("Shows a topic."),
        )
        .subcommand(Command::new("debug").hide(true).about("Dumps state."))
}

fn help() -> Help {
    Help::new(tool())
        .topic::<SignIn>()
        .topic::<Retention>()
        .table::<Refusal>("refusal")
        .constant("MAX_PLANS", 10)
        .constant("RETENTION", "for 30 days")
}

fn faults(error: agent_help::Error) -> Vec<String> {
    error.faults().iter().map(Fault::to_string).collect()
}

#[test]
fn the_manual_lists_every_command_by_group_then_every_topic() {
    assert_eq!(
        help().manual().unwrap(),
        "\
# `plans`

Keeps plans, at most 10.

Run `plans plan list` first.

## Usage

```
plans [COMMAND]
```

## Commands

### Run by the agent

#### `plans plan`

Works with plans.

##### Usage

```
plans plan [COMMAND]
```

#### `plans plan create`

Creates a plan.

It may refuse:

- `name_taken`: The name is taken by another plan.
- `over_limit`: There are 10 plans already.

##### Usage

```
plans plan create
```

#### `plans plan list`

Lists every plan.

##### Usage

```
plans plan list
```

#### `plans help`

Shows a topic.

##### Usage

```
plans help
```

### Run by the operator

#### `plans serve`

Serves the plans.

##### Usage

```
plans serve
```

## Topics

### `plans help sign-in`

Run `plans login` once; it keeps the token in the keychain.

### `plans help retention`

A plan is kept for 30 days.
"
    );
}

#[test]
fn the_index_lists_each_command_s_line_by_group_then_each_topic_s() {
    assert_eq!(
        help().index().unwrap(),
        "\
# `plans`

Keeps plans, at most 10.

Run `plans plan list` first.

## Usage

```
plans [COMMAND]
```

## Commands

### Run by the agent

- `plans plan`: Works with plans.
- `plans plan create`: Creates a plan.
- `plans plan list`: Lists every plan.
- `plans help`: Shows a topic.

### Run by the operator

- `plans serve`: Serves the plans.

## Topics

- `plans help sign-in`: Run `plans login` once; it keeps the token in the keychain.
- `plans help retention`: A plan is kept for 30 days.
- `plans help all`: The whole manual: every command and every topic, in full.
"
    );
}

#[test]
fn a_command_s_line_is_the_first_line_of_its_about() {
    let help = Help::new(
        Command::new("plans")
            .about("Keeps plans.")
            .subcommand(
                Command::new("serve")
                    .next_help_heading("Run by the operator")
                    .about("Serves {const:WHAT}.\nUntil stopped."),
            )
            .subcommand(
                Command::new("stop")
                    .next_help_heading("Run by the operator")
                    .long_about("Stops serving.\n\nAt once."),
            ),
    )
    .constant("WHAT", "the plans");
    let index = help.index().unwrap();
    let commands = index.split("## Commands\n").nth(1).unwrap();
    assert_eq!(
        commands,
        "\n### Run by the operator\n\n\
         - `plans serve`: Serves the plans.\n\
         - `plans stop`: Stops serving.\n\n\
         ## Topics\n\n\
         - `plans help all`: The whole manual: every command and every topic, in full.\n"
    );
}

#[test]
fn a_topic_s_line_is_the_first_sentence_of_its_markdown() {
    /// Sign in once. Run `plans login` (it asks `y/n?` once)!
    #[derive(Topic)]
    #[topic(name = "sign-in")]
    struct Long;
    let index = Help::new(Command::new("plans").about("Keeps plans."))
        .topic::<Long>()
        .index()
        .unwrap();
    assert!(
        index.contains("- `plans help sign-in`: Sign in once.\n"),
        "{index}"
    );
}

#[test]
fn the_index_answers_only_when_the_whole_manual_does() {
    let help = help().topic::<SignIn>();
    assert_eq!(
        help.index().unwrap_err(),
        help.manual().unwrap_err(),
        "a fault of the whole manual"
    );
}

#[test]
fn help_all_answers_the_whole_manual() {
    assert_eq!(help().topic_help("all"), help().manual());
}

#[test]
fn a_nested_command_may_declare_a_group_of_its_own() {
    let help = Help::new(
        Command::new("plans").subcommand(
            Command::new("plan")
                .next_help_heading("Run by the agent")
                .about("Plans.")
                .subcommand(
                    Command::new("purge")
                        .next_help_heading("Run by the operator")
                        .about("Purges."),
                ),
        ),
    );
    assert_eq!(
        help.manual().unwrap(),
        "\
# `plans`

## Usage

```
plans [COMMAND]
```

## Commands

### Run by the agent

#### `plans plan`

Plans.

##### Usage

```
plans plan [COMMAND]
```

### Run by the operator

#### `plans plan purge`

Purges.

##### Usage

```
plans plan purge
```
"
    );
}

#[test]
fn a_subcommand_with_no_group_is_an_error_naming_every_such_one() {
    let help = Help::new(
        Command::new("plans")
            .subcommand(Command::new("serve").about("Serves."))
            .subcommand(Command::new("stop").about("Stops.")),
    );
    assert_eq!(
        faults(help.manual().unwrap_err()),
        [
            "`plans serve` has no group: declare who runs it where it is defined, \
             with `#[command(next_help_heading = \"...\")]`",
            "`plans stop` has no group: declare who runs it where it is defined, \
             with `#[command(next_help_heading = \"...\")]`",
        ]
    );
}

#[test]
fn one_command_can_be_asked_for_alone() {
    assert_eq!(
        help().command_help(&["plan", "create"]).unwrap(),
        "\
# `plans plan create`

Creates a plan.

It may refuse:

- `name_taken`: The name is taken by another plan.
- `over_limit`: There are 10 plans already.

## Usage

```
plans plan create
```
"
    );
    assert_eq!(
        help().command_help(&[]).unwrap(),
        help().index().unwrap(),
        "the tool's own command is the index"
    );
}

#[test]
fn an_unknown_command_is_an_error_naming_the_commands_there_are() {
    assert_eq!(
        help()
            .command_help(&["plan", "remove"])
            .unwrap_err()
            .to_string(),
        "no command `plans plan remove`; the commands are: `plans plan`, \
         `plans plan create`, `plans plan list`, `plans serve`, `plans help`"
    );
}

#[test]
fn one_topic_can_be_asked_for_alone() {
    assert_eq!(
        help().topic_help("retention").unwrap(),
        "# `plans help retention`\n\nA plan is kept for 30 days.\n"
    );
}

#[test]
fn an_unknown_topic_is_an_error_naming_the_topics_there_are() {
    assert_eq!(
        help().topic_help("billing").unwrap_err().to_string(),
        "no topic `billing`; the topics are: `sign-in`, `retention`"
    );
}

#[test]
fn a_placeholder_naming_nothing_registered_is_an_error_naming_it() {
    let help = Help::new(
        Command::new("plans").long_about("At most {const:MAX}.\n\n{table:codes}\n\n{const:MAX}"),
    )
    .table::<Refusal>("refusal")
    .constant("MAX_PLANS", 10);
    assert_eq!(
        faults(help.manual().unwrap_err()),
        [
            "`{const:MAX}` in `plans` names no constant; the constants are: `MAX_PLANS`",
            "`{table:codes}` in `plans` names no table; the tables are: `refusal`",
        ]
    );
}

#[test]
fn a_placeholder_in_a_table_or_a_topic_is_checked_where_it_is_written() {
    let help = Help::new(Command::new("plans").long_about("{table:refusal}"))
        .topic::<Retention>()
        .table::<Refusal>("refusal");
    assert_eq!(
        faults(help.manual().unwrap_err()),
        [
            "`{const:MAX_PLANS}` in table `refusal` names no constant; the constants are: none",
            "`{const:RETENTION}` in topic `retention` names no constant; \
             the constants are: none",
        ]
    );
}

#[test]
fn a_table_does_not_include_a_table() {
    /// Codes.
    #[derive(agent_help::Help)]
    #[allow(dead_code)]
    enum Nested {
        /// See {table:nested}.
        Loop,
    }
    let help =
        Help::new(Command::new("plans").long_about("{table:nested}")).table::<Nested>("nested");
    assert_eq!(
        faults(help.manual().unwrap_err()),
        ["`{table:nested}` in table `nested`: a table's help names constants, not tables"]
    );
}

#[test]
fn text_that_only_looks_like_a_placeholder_is_kept() {
    let help = Help::new(
        Command::new("plans").long_about("```rust\nlet x = {table};\nformat!(\"{const: }\")\n```"),
    );
    assert_eq!(
        help.manual().unwrap(),
        "# `plans`\n\n```rust\nlet x = {table};\nformat!(\"{const: }\")\n```\n\n\
         ## Usage\n\n```\nplans\n```\n"
    );
}

#[test]
fn a_topic_of_another_binary_is_refused() {
    let help = Help::new(Command::new("plans")).topic::<ForeignSignIn>();
    let refused = "topic `sign-in` is shown by `bk-server help sign-in`, but this tool is \
                   `plans`: name it `agent_help::binary!(\"plans\")`";
    assert_eq!(faults(help.manual().unwrap_err()), [refused]);
    assert_eq!(help.topic_help("sign-in").unwrap_err().to_string(), refused);
}

#[test]
fn clap_answers_every_help_flag_from_the_manual() {
    let help = help();
    let answer = |args: &[&str]| {
        let error = help.clap().unwrap().try_get_matches_from(args).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::DisplayHelp, "{args:?}");
        error.to_string()
    };
    assert_eq!(answer(&["plans", "--help"]), help.index().unwrap());
    assert_eq!(answer(&["plans", "-h"]), help.index().unwrap());
    assert_eq!(
        answer(&["plans", "plan", "create", "--help"]),
        help.command_help(&["plan", "create"]).unwrap()
    );
    assert_eq!(
        answer(&["plans", "serve", "-h"]),
        help.command_help(&["serve"]).unwrap()
    );
}

#[test]
fn clap_refuses_a_broken_manual_naming_every_fault() {
    let help = Help::new(
        Command::new("plans")
            .subcommand(Command::new("serve"))
            .subcommand(Command::new("debug").hide(true).about("{const:X}")),
    );
    assert_eq!(
        faults(help.clap().unwrap_err()),
        [
            "`plans serve` has no group: declare who runs it where it is defined, \
             with `#[command(next_help_heading = \"...\")]`",
            "`{const:X}` in `plans debug` names no constant; the constants are: none",
        ]
    );
}
