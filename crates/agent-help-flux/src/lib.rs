//! A pure crate as a tool writes one, checked by Flux: it derives `Help` and
//! `Topic`, so `cargo flux` on it checks the code those derives generate in a
//! crate other than agent-help's. Its tests are agent-help's: this crate
//! only has to pass Flux with nothing trusted or ignored.

use agent_help::{Help, Topic};

agent_help::binary!("tool");

/// Why `tool` refused a request.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Help)]
pub enum Refusal {
    /// The name is taken by another plan.
    NameTaken,
    /// The plan is over its limit.
    OverLimit { by: u32 },
    /// The server did not answer.
    #[help = "The server did not answer in time: run the command again."]
    Timeout(u64),
}

/// How to sign in.
#[derive(Topic)]
#[topic(name = "sign-in")]
pub struct SignIn;
