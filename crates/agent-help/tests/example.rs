//! The example tool, `examples/plans.rs`, built and run as an agent runs it.

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

/// The example's binary, built once for every test here.
fn plans() -> Command {
    static BUILT: OnceLock<PathBuf> = OnceLock::new();
    let binary = BUILT.get_or_init(|| {
        let status = Command::new(env!("CARGO"))
            .args([
                "build",
                "--quiet",
                "--package",
                "agent-help",
                "--example",
                "plans",
            ])
            .status()
            .expect("cargo runs");
        assert!(status.success(), "the example builds");
        // This test runs from `<target>/<profile>/deps`; examples are built
        // beside it, in `<target>/<profile>/examples`.
        let deps = std::env::current_exe().unwrap();
        let profile = deps.parent().unwrap().parent().unwrap();
        profile
            .join("examples")
            .join(format!("plans{}", std::env::consts::EXE_SUFFIX))
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

const CREATE: &str = "\
# `plans plan create`

Creates a plan.

Answers `{\"plan\": \"<name>\"}`. It may refuse:

- `name_taken`: The name is taken by another plan.
- `over_limit`: There are 10 plans already: remove one first.
";

#[test]
fn help_answers_the_whole_manual() {
    let manual = "\
# `plans`

Keeps plans for a team, at most 10.

Every command prints one JSON object. Run `plans plan list` first.

## Commands

### Run by the agent

#### `plans plan`

Works with plans.

#### `plans plan create`

Creates a plan.

Answers `{\"plan\": \"<name>\"}`. It may refuse:

- `name_taken`: The name is taken by another plan.
- `over_limit`: There are 10 plans already: remove one first.

#### `plans plan list`

Lists every plan.

#### `plans help`

Shows one topic of this manual, or all of it.

`plans help sign-in` shows how to sign in.

### Run by the operator

#### `plans serve`

Serves the plans to the team.

Run it once, where the store is; it answers until stopped.

## Topics

### `plans help sign-in`

Run `plans login` once; it keeps the token in the keychain.

Every command refuses with `not_signed_in` until then.
";
    assert_eq!(run(&["--help"]), (manual.to_owned(), true));
    assert_eq!(run(&["help"]), (manual.to_owned(), true));
}

#[test]
fn a_command_s_help_answers_that_command() {
    assert_eq!(
        run(&["plan", "create", "--help"]),
        (CREATE.to_owned(), true)
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
