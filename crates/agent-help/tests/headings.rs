//! A command's or topic's sections, written from `##` as on a page of its
//! own, nested under wherever the manual places its text.

use agent_help::{Fault, Help, Topic};
use clap::Command;

agent_help::binary!("plans");

/// Run `plans login` once.
///
/// ## When
///
/// Before any other command.
///
/// ### On a server
///
/// Set `PLANS_TOKEN` instead.
#[derive(Topic)]
#[topic(name = "sign-in")]
struct SignIn;

/// # Signing in
///
/// Run `plans login` once.
#[derive(Topic)]
#[topic(name = "titled")]
struct Titled;

/// Why `plans` refused a request.
#[derive(agent_help::Help)]
#[allow(dead_code)]
enum Refusal {
    /// The plan is over its limit.
    ///
    /// ## Fix
    ///
    /// Raise the limit.
    OverLimit,
}

const CREATE: &str = "\
Creates a plan.

## When

Once per plan.

```md
## Not a heading
```

~~~
# Nor this
~~~

#hashtag, ####### seven, and    ## indented four are not headings either.

## Trips

It may refuse:

{table:refusal}";

fn tool(create: &str) -> Command {
    Command::new("plans").about("Keeps plans.").subcommand(
        Command::new("plan")
            .next_help_heading("Run by the agent")
            .about("Works with plans.")
            .subcommand(Command::new("create").long_about(create.to_owned())),
    )
}

fn help() -> Help {
    Help::new(tool(CREATE))
        .topic::<SignIn>()
        .table::<Refusal>("refusal")
}

fn faults(error: agent_help::Error) -> Vec<String> {
    error.faults().iter().map(Fault::to_string).collect()
}

#[test]
fn a_command_alone_keeps_its_sections_as_written() {
    assert_eq!(
        help().command_help(&["plan", "create"]).unwrap(),
        "\
# `plans plan create`

Creates a plan.

## When

Once per plan.

```md
## Not a heading
```

~~~
# Nor this
~~~

#hashtag, ####### seven, and    ## indented four are not headings either.

## Trips

It may refuse:

- `over_limit`: The plan is over its limit.

  ## Fix

  Raise the limit.
"
    );
}

#[test]
fn a_topic_alone_keeps_its_sections_as_written() {
    assert_eq!(
        help().topic_help("sign-in").unwrap(),
        "\
# `plans help sign-in`

Run `plans login` once.

## When

Before any other command.

### On a server

Set `PLANS_TOKEN` instead.
"
    );
}

#[test]
fn in_the_manual_each_heading_nests_under_its_command_or_topic() {
    assert_eq!(
        help().manual().unwrap(),
        "\
# `plans`

Keeps plans.

## Commands

### Run by the agent

#### `plans plan`

Works with plans.

#### `plans plan create`

Creates a plan.

##### When

Once per plan.

```md
## Not a heading
```

~~~
# Nor this
~~~

#hashtag, ####### seven, and    ## indented four are not headings either.

##### Trips

It may refuse:

- `over_limit`: The plan is over its limit.

  ##### Fix

  Raise the limit.

## Topics

### `plans help sign-in`

Run `plans login` once.

#### When

Before any other command.

##### On a server

Set `PLANS_TOKEN` instead.
"
    );
}

#[test]
fn a_heading_nested_past_six_levels_is_an_error_naming_where_it_is() {
    let help = Help::new(tool("Creates a plan.\n\n### Steps\n\n#### Step one"));
    let deep = [
        "`#### Step one` in `plans plan create` is nested past `######` in the manual: \
         write it `###` at most",
    ];
    assert_eq!(faults(help.manual().unwrap_err()), deep);
    assert_eq!(faults(agent_help::check(&help).unwrap_err()), deep);
    assert!(
        help.command_help(&["plan", "create"]).is_ok(),
        "alone, it is not nested past `######`"
    );
}

#[test]
fn a_title_heading_in_a_text_is_an_error_naming_where_it_is() {
    let help = Help::new(tool("# Create\n\nCreates a plan.")).topic::<Titled>();
    let titled = [
        "`# Create` in `plans plan create` is a title, and the manual writes the title: \
         start the text's sections from `##`",
        "`# Signing in` in topic `titled` is a title, and the manual writes the title: \
         start the text's sections from `##`",
    ];
    assert_eq!(faults(help.manual().unwrap_err()), titled);
    assert_eq!(
        faults(help.command_help(&["plan", "create"]).unwrap_err()),
        titled[..1]
    );
    assert_eq!(faults(help.topic_help("titled").unwrap_err()), titled[1..]);
}

#[test]
fn a_heading_in_a_table_is_nested_and_checked_with_the_text_it_is_in() {
    /// Codes.
    #[derive(agent_help::Help)]
    #[allow(dead_code)]
    enum Titled {
        /// Over.
        ///
        /// # Fix
        Over,
    }
    let help = Help::new(tool("Creates a plan.\n\n{table:codes}")).table::<Titled>("codes");
    assert_eq!(
        faults(help.manual().unwrap_err()),
        [
            "`# Fix` in `plans plan create` is a title, and the manual writes the title: \
             start the text's sections from `##`"
        ]
    );
}

#[test]
fn code_blocks_fenced_with_tildes_are_handed_over_too() {
    let help = Help::new(tool("Creates a plan.\n\n~~~sh\nplans plan create q3\n~~~"));
    assert_eq!(
        agent_help::code_blocks(&help, "sh")
            .into_iter()
            .map(|block| block.code)
            .collect::<Vec<_>>(),
        ["plans plan create q3\n"]
    );
}
