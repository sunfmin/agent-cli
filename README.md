# agent-cli

A Claude Code skill: how to design and build command-line tools in Rust whose first reader is an AI agent.

```
npx skills add sunfmin/agent-cli -g
```

The skill is [`skills/agent-cli/SKILL.md`](skills/agent-cli/SKILL.md).

`crates/agent-help` is a library for such tools' help: written where each
fact lives, assembled into one Markdown manual. `./gate.sh` is what must pass
before a change lands: fmt, Flux on every crate that enables it
(`./verify.sh`, which needs the `cargo-flux`, `fixpoint` and `z3` that
`Cargo.toml` pins, and says how to install each), clippy and nextest, on the
nightly `gate.sh` names; the fleet works the backlog as
[`docs/agents/afkconvoy.md`](docs/agents/afkconvoy.md) says.
