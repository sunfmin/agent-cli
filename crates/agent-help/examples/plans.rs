//! A tool wired to its manual: `plans --help` answers the whole manual,
//! `plans <command> --help` one command, `plans help <topic>` one topic.
//!
//! ```sh
//! cargo run --example plans -- --help
//! ```

use std::process::ExitCode;

use agent_help::{Help, Topic};
use clap::{FromArgMatches, Parser, Subcommand};

agent_help::binary!("plans");

/// The most plans a store keeps.
const MAX_PLANS: u32 = 10;

/// Keeps plans for a team, at most {const:MAX_PLANS}.
///
/// Every command prints one JSON object. Run `plans plan list` first.
#[derive(Parser)]
#[command(name = "plans", verbatim_doc_comment, disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Works with plans.
    #[command(
        subcommand,
        next_help_heading = "Run by the agent",
        verbatim_doc_comment
    )]
    Plan(PlanCommand),
    /// Shows one topic of this manual, or all of it.
    ///
    /// `plans help sign-in` shows how to sign in.
    #[command(next_help_heading = "Run by the agent", verbatim_doc_comment)]
    Help {
        /// The topic, such as `sign-in`.
        topic: Option<String>,
    },
    /// Serves the plans to the team.
    ///
    /// Run it once, where the store is; it answers until stopped.
    #[command(next_help_heading = "Run by the operator", verbatim_doc_comment)]
    Serve,
}

#[derive(Subcommand)]
enum PlanCommand {
    /// Creates a plan.
    ///
    /// Answers `{"plan": "<name>"}`. It may refuse:
    ///
    /// {table:refusal}
    #[command(verbatim_doc_comment)]
    Create {
        /// The plan's name.
        name: String,
    },
    /// Lists every plan.
    #[command(verbatim_doc_comment)]
    List,
}

/// Why `plans` refused a request.
#[derive(agent_help::Help)]
#[allow(dead_code)] // this stub of a tool never refuses
enum Refusal {
    /// The name is taken by another plan.
    NameTaken,
    /// There are {const:MAX_PLANS} plans already: remove one first.
    OverLimit,
}

/// Run `plans login` once; it keeps the token in the keychain.
///
/// Every command refuses with `not_signed_in` until then.
#[derive(Topic)]
#[topic(name = "sign-in")]
struct SignIn;

fn help() -> Help {
    Help::new(<Cli as clap::CommandFactory>::command())
        .topic::<SignIn>()
        .table::<Refusal>("refusal")
        .constant("MAX_PLANS", MAX_PLANS)
}

fn main() -> ExitCode {
    let help = help();
    let command = match help.clap() {
        Ok(command) => command,
        Err(error) => return refuse(&error.to_string()),
    };
    let cli = match command
        .try_get_matches()
        .and_then(|matches| Cli::from_arg_matches(&matches))
    {
        Ok(cli) => cli,
        // `--help` too: clap prints the manual's answer, on stdout.
        Err(error) => error.exit(),
    };
    match cli.command {
        Command::Help { topic: None } => print!("{}", help.manual().expect("checked by clap()")),
        Command::Help { topic: Some(topic) } => match help.topic_help(&topic) {
            Ok(markdown) => print!("{markdown}"),
            Err(error) => return refuse(&error.to_string()),
        },
        Command::Plan(PlanCommand::Create { name }) => {
            println!("{{\"plan\": \"{name}\"}}");
        }
        Command::Plan(PlanCommand::List) => println!("{{\"plans\": []}}"),
        Command::Serve => println!("{{\"serving\": true}}"),
    }
    ExitCode::SUCCESS
}

/// The tool's failure: one JSON object on stdout, and a non-zero exit.
fn refuse(error: &str) -> ExitCode {
    let error = error
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    println!("{{\"error\": \"{error}\"}}");
    ExitCode::FAILURE
}
