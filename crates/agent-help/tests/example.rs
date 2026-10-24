//! The example tool, `examples/plans.rs`, built and run as an agent runs it.

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

/// The example's binary, built once for every test here.
fn plans() -> Command {
    static BUILT: OnceLock<PathBuf> = OnceLock::new();
    let binary = BUILT.get_or_init(|| {
        let output = Command::new(env!("CARGO"))
            .args([
                "build",
                "--quiet",
                "--package",
                "agent-help",
                "--example",
                "plans",
                "--message-format=json",
            ])
            .stderr(std::process::Stdio::inherit())
            .output()
            .expect("cargo runs");
        assert!(output.status.success(), "the example builds");
        // Where the binary lands is cargo's to say, and differs between
        // cargo versions: its message about the example names it.
        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
            .find(|message| {
                message["reason"] == "compiler-artifact" && message["target"]["name"] == "plans"
            })
            .and_then(|artifact| artifact["executable"].as_str().map(PathBuf::from))
            .expect("cargo names the example's binary")
    });
    Command::new(binary)
}

/// What `plans <args>` prints on stdout, and whether it succeeded.
fn run(args: &[&str]) -> (String, bool) {
    let output = plans().args(args).output().expect("the example runs");
    (
        String::from_utf8(output.stdout).unwrap(),
        output.status.success(),
    )
}

const SIGN_IN: &str = "\
# `plans help sign-in`

Run `plans login` once; it keeps the token in the keychain.

Every command refuses with `not_signed_in` until then.
";

/// The global `--store`, as every command lists it.
const STORE: &str = "\
- `--store <PATH>`: Where the store is.
  - Default: `./plans`
  - Environment: `PLANS_STORE`
";

/// `plans plan create`'s answer, its headings `#` deep and their sections one
/// deeper.
fn create(heading: &str) -> String {
    format!(
        "\
{heading} `plans plan create`

Creates a plan.

Answers `{{\"plan\": \"<name>\"}}`. It may refuse:

- `name_taken`: The name is taken by another plan.
- `over_limit`: There are 10 plans already: remove one first.

{heading}# Usage

```
plans plan create [OPTIONS] <NAME>
```

{heading}# Arguments

- `<NAME>`: The plan's name.
{STORE}"
    )
}

#[test]
fn help_answers_the_whole_manual() {
    let create = create("####");
    let manual = format!(
        "\
# `plans`

Keeps plans for a team, at most 10.

Every command prints one JSON object. Run `plans plan list` first.

## Usage

```
plans [OPTIONS] <COMMAND>
```

## Arguments

{STORE}
## Commands

### Run by the agent

#### `plans plan`

Works with plans.

##### Usage

```
plans plan [OPTIONS] <COMMAND>
```

##### Arguments

{STORE}
{create}
#### `plans plan list`

Lists every plan.

##### Usage

```
plans plan list [OPTIONS]
```

##### Arguments

- `--format <FORMAT>`: How the list is written.
  - Default: `json`
  - Values:
    - `json`: One JSON object.
    - `markdown`: A Markdown table.
{STORE}
#### `plans help`

Shows one topic of this manual, or all of it.

`plans help sign-in` shows how to sign in.

##### Usage

```
plans help [OPTIONS] [TOPIC]
```

##### Arguments

- `[TOPIC]`: The topic, such as `sign-in`.
{STORE}
### Run by the operator

#### `plans serve`

Serves the plans to the team.

Run it once, where the store is; it answers until stopped.

##### Usage

```
plans serve [OPTIONS]
```

##### Arguments

{STORE}
## Topics

### `plans help sign-in`

Run `plans login` once; it keeps the token in the keychain.

Every command refuses with `not_signed_in` until then.
"
    );
    assert_eq!(run(&["--help"]), (manual.clone(), true));
    assert_eq!(run(&["help"]), (manual, true));
}

#[test]
fn a_command_s_help_answers_that_command() {
    assert_eq!(run(&["plan", "create", "--help"]), (create("#"), true));
}

#[test]
fn an_argument_s_default_and_environment_variable_are_what_clap_uses() {
    assert_eq!(
        run(&["plan", "list"]),
        ("{\"store\": \"./plans\", \"plans\": []}\n".to_owned(), true)
    );
    let output = plans()
        .args(["plan", "list"])
        .env("PLANS_STORE", "/srv/plans")
        .output()
        .expect("the example runs");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "{\"store\": \"/srv/plans\", \"plans\": []}\n"
    );
}

#[test]
fn help_with_a_topic_answers_that_topic() {
    assert_eq!(run(&["help", "sign-in"]), (SIGN_IN.to_owned(), true));
}

#[test]
fn help_with_an_unknown_topic_fails_naming_the_topics() {
    assert_eq!(
        run(&["help", "billing"]),
        (
            "{\"error\": \"no topic `billing`; the topics are: `sign-in`\"}\n".to_owned(),
            false
        )
    );
}
