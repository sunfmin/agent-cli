# agent-cli

A Claude Code skill: how to design and build command-line tools in Rust whose first reader is an AI agent.

```
npx skills add sunfmin/agent-cli -g
```

The skill is [`skills/agent-cli/SKILL.md`](skills/agent-cli/SKILL.md).

## agent-help

`crates/agent-help` is the library the skill writes help with: Markdown doc
comments on the types each fact belongs to (clap's commands, an enum of
refusal codes, a topic), assembled into one manual that answers `--help`,
`<command> --help` and `help <topic>`, and `agent_help::check`, the one test
that keeps it whole. `cargo doc -p agent-help --open` says how;
`crates/agent-help/examples/plans.rs` is a tool wired to it.

It has no release yet: a tool depends on it from git, pinned to a commit,
beside clap with its derive:

```toml
[dependencies]
agent-help = { git = "https://github.com/sunfmin/agent-cli", rev = "<commit>" }
clap = { version = "4.5", features = ["derive"] }
```

## Working on this repo

A tool's pure crates pass Flux, so what agent-help's derives generate does
too: `crates/agent-help-flux` derives them in a crate Flux checks. `./gate.sh` is what must pass before a change
lands: fmt, Flux on every crate that enables it, each required to report
`0 trusted; 0 ignored` (`./verify.sh`, which needs the `cargo-flux`,
`fixpoint` and `z3` that `Cargo.toml` pins, and says how to install each),
clippy and nextest, on the nightly `gate.sh` names; the fleet works the
backlog as
[`docs/agents/afkconvoy.md`](docs/agents/afkconvoy.md) says.
