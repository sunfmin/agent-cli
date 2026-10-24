//! A command's usage and arguments, read from clap's tree into its answer.

use agent_help::{Fault, Help};
use clap::builder::PossibleValue;
use clap::{Arg, ArgAction, Command};

fn tool() -> Command {
    Command::new("plans")
        .version("1.0")
        .disable_help_subcommand(true)
        .about("Keeps plans.")
        .arg(
            Arg::new("store")
                .long("store")
                .short('s')
                .global(true)
                .value_name("PATH")
                .default_value("./plans")
                .env("PLANS_STORE")
                .help("Where the store is."),
        )
        .arg(
            Arg::new("token")
                .long("token")
                .hide(true)
                .help("Signs every request."),
        )
        .subcommand(
            Command::new("create")
                .next_help_heading("Run by the agent")
                .about("Creates a plan.")
                .arg(
                    Arg::new("name")
                        .required(true)
                        .help("The plan's name.")
                        .long_help("The plan's name.\n\nAt most {const:MAX_NAME} characters:\n\n```sh\nplans create q3\n```"),
                )
                .arg(Arg::new("tags").num_args(1..).help("Its tags."))
                .arg(
                    Arg::new("format")
                        .long("format")
                        .help("How the answer is written.")
                        .default_values(["json"])
                        .value_parser([
                            PossibleValue::new("json").help("One JSON object."),
                            PossibleValue::new("markdown"),
                            PossibleValue::new("yaml").hide(true),
                        ]),
                )
                .arg(
                    Arg::new("lang")
                        .long("lang")
                        .help("The plan's language.")
                        .value_parser(["go", "rust"])
                        .hide_default_value(true)
                        .default_value("go"),
                )
                .arg(
                    Arg::new("dry-run")
                        .long("dry-run")
                        .action(ArgAction::SetTrue)
                        .help("Answers without writing."),
                )
                .arg(Arg::new("quiet").short('q').action(ArgAction::SetTrue)),
        )
        .subcommand(
            Command::new("serve")
                .next_help_heading("Run by the operator")
                .about("Serves the plans.")
                .subcommand(Command::new("stop").about("Stops serving.")),
        )
}

fn help() -> Help {
    Help::new(tool()).constant("MAX_NAME", 40)
}

const STORE: &str = "\
- `-s, --store <PATH>`: Where the store is.
  - Default: `./plans`
  - Environment: `PLANS_STORE`
";

#[test]
fn a_command_s_answer_lists_its_usage_then_its_arguments_positional_first() {
    assert_eq!(
        help().command_help(&["create"]).unwrap(),
        format!(
            "\
# `plans create`

Creates a plan.

## Usage

```
plans create [OPTIONS] <name> [tags]...
```

## Arguments

- `<name>`: The plan's name.

  At most 40 characters:

  ```sh
  plans create q3
  ```
- `[tags]...`: Its tags.
- `--format <format>`: How the answer is written.
  - Default: `json`
  - Values:
    - `json`: One JSON object.
    - `markdown`
- `--lang <lang>`: The plan's language.
  - Values:
    - `go`
    - `rust`
- `--dry-run`: Answers without writing.
- `-q`
{STORE}"
        )
    );
}

#[test]
fn a_global_argument_is_listed_on_every_command_it_reaches() {
    assert_eq!(
        help().command_help(&["serve", "stop"]).unwrap(),
        format!(
            "\
# `plans serve stop`

Stops serving.

## Usage

```
plans serve stop [OPTIONS]
```

## Arguments

{STORE}"
        )
    );
}

#[test]
fn a_command_with_no_arguments_has_no_arguments_section() {
    let help = Help::new(
        Command::new("plans").subcommand(
            Command::new("list")
                .next_help_heading("Run by the agent")
                .about("Lists every plan."),
        ),
    );
    assert_eq!(
        help.command_help(&["list"]).unwrap(),
        "\
# `plans list`

Lists every plan.

## Usage

```
plans list
```
"
    );
}

#[test]
fn the_manual_lists_the_tool_s_arguments_and_each_command_s() {
    let manual = help().manual().unwrap();
    let tool = format!(
        "\
# `plans`

Keeps plans.

## Usage

```
plans [OPTIONS] [COMMAND]
```

## Arguments

{STORE}
## Commands
"
    );
    assert!(manual.starts_with(&tool), "{manual}");
    let stop = format!(
        "\
#### `plans serve stop`

Stops serving.

##### Usage

```
plans serve stop [OPTIONS]
```

##### Arguments

{STORE}"
    );
    assert!(manual.ends_with(&stop), "{manual}");
}

#[test]
fn a_command_s_place_in_the_manual_is_its_answer_one_level_down() {
    let manual = help().manual().unwrap();
    let alone = help().command_help(&["create"]).unwrap();
    let entry = alone
        .lines()
        .map(|line| match line.strip_prefix('#') {
            Some(heading) if line.starts_with("# ") || line.starts_with("## ") => {
                format!("####{heading}")
            }
            _ => line.to_owned(),
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(manual.contains(&entry), "{manual}\n---\n{entry}");
}

#[test]
fn a_placeholder_in_an_argument_s_help_is_checked_where_it_is_written() {
    let help = Help::new(tool());
    assert_eq!(
        help.command_help(&["create"])
            .unwrap_err()
            .faults()
            .iter()
            .map(Fault::to_string)
            .collect::<Vec<_>>(),
        [
            "`{const:MAX_NAME}` in `<name>` of `plans create` names no constant; \
          the constants are: none"
        ]
    );
}
