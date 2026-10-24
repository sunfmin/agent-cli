---
name: agent-cli
description: How every command-line tool (CLI) is built here — in Rust, read first by an AI agent, with pure crates checked by Flux, a thin effect layer, help as the only manual, one JSON object on stdout, no state in the process, released for macOS, Linux and Windows through Homebrew and Scoop. Use whenever the user wants to build or design a new CLI or command-line tool, whatever it does and whether or not they mention agents or Rust, wants a new subcommand, or is deciding a CLI's output, errors, help or release. Load it together with any general design skill, since it fixes the language and structure those leave open.
---

# agent-cli

An agent knows a tool by three things: its `--help`, its stdout, its exit code.
Those are the interface. Every rule below exists to make them trustworthy.

## What we believe

1. **Decide purely, act thinly.** A decision is a function of its arguments and can
   be proved. An action touches the world and can only be tested. Keep them apart
   and make the second one small.
2. **A rule is a type, not a habit.** What a checker can refuse, nobody has to
   remember.
3. **One home per fact.** Everything else is derived from it or tested against it.
4. **The process holds nothing.** Truth lives in the world. Read it again each run.
5. **Answer; never ask, never help yourself.** Install nothing, log nobody in,
   prompt for nothing. Refuse, and give the line that fixes it.
6. **The reader is literal.** It does not guess, read source, or see colour.

## Rust, always

One cargo workspace.

- **Pure crates**, as many as the domain has parts. No I/O. Clock, environment and
  randomness arrive as arguments. `publish = false`.
- **One effect crate**, the binary. It reads the world, asks the pure crates what
  to do, and does it. `main.rs` is one line calling `main(args) -> ExitCode`.

A crate is pure or it is not: no crate is mostly pure.

Dependencies stay few: `clap` (derive, env), `agent-help`, `anyhow`, `serde`,
`serde_json`.
Reach other systems through their own CLIs, one module per command, the only
place that builds its command line. No SDKs, no tokens handled.

## Pure crates pass Flux

Every function of every pure crate is checked by Flux refinement types.

- Refine two kinds of rule: **bounds** (never negative, never past a limit) and
  **order** (this step only after that one; this step last).
- No exemptions. The gate reads Flux's summary and requires `0 trusted; 0 ignored`.
- Pin Flux, `fixpoint` and `z3` in `Cargo.toml`. `verify.sh` reads the pins and
  refuses any other checker, naming how to install the right one.
- Flux attributes expand to nothing in a plain build: the gate runs nightly, the
  release builds on stable.
- Know its edge. Flux proves what a call requires, not that the call happens:
  make the last step the only thing that can produce the return value. And the
  proof starts at what the effect crate reports, not at the world.
- What Flux cannot say, say with `proptest`.

## The output contract

- Every command prints exactly one JSON object on stdout.
- A failure is `{"error": "…"}` and a non-zero exit. Warnings go to stderr.
- An expected outcome is an answer with exit 0, not an error.
- An error names every fault at once, each with the line that fixes it.
- Any exception to these is named in the top-level help.

## Help is the only manual

Written with [agent-help](https://github.com/sunfmin/agent-cli/tree/main/crates/agent-help);
its crate docs say how.

- Help is Markdown, written as doc comments on the type that is each fact's
  one home: a subcommand's clap struct or variant, a refusal code's enum
  variant, a topic's type. It is assembled when the tool is built; nothing is
  copied by hand.
- The types are the registries: clap's command tree and an enum's variants,
  never a link-time collection. A fact left out is a failure, never a silent
  gap: a variant without help does not compile, a command without text fails
  `check`.
- The answer is the Markdown as written: no rendering, no wrapping, no width.
  Write each paragraph on one line.
- `--help` answers the whole manual, `<command> --help` that command; `-h`
  answers what `--help` does. What belongs to no command (how to sign in, a
  concept every command shares) is a topic: `help <topic>`.
- The manual says: what it is, what it needs, what to run first, the commands
  grouped by who runs them, the rules to know first.
- A command's help says: when to run it, what it reads and writes, what it
  answers (every key), what it may refuse, what trips you up. Who runs it is
  its group, declared where it is defined.
- A number or a code list is written once, in code, and placed in help by a
  placeholder. A message names a topic by its reference, never by its name
  written out.
- Help names nothing its reader cannot open: no ADR numbers, no repository
  paths, no internal documents. The reader has the binary and its help,
  nothing else.
- The shipped SKILL.md is a few lines that send the agent to `--help`.
- `agent_help::check` is the one test that keeps the manual whole: every
  command has text and a group, every placeholder and topic link names
  something.

## Nothing in the process

- No daemon state, no cache that is the truth, no file only this tool understands.
- Every write is idempotent: run twice, same world.
- Kill it anywhere and the next run finishes the job from what it reads.

## Every tool has

- `doctor`: reads everything, changes nothing, names the fix of each finding.
- A command that says what would happen and changes nothing.
- `--version` stamped by `build.rs`: `1.2.3` for a release, otherwise
  `1.2.3+<commits>.g<commit>[.dirty]`.

## Tests

- Run the real binary; assert its JSON and exit code.
- Stand in for each external CLI with a cargo example placed first on `PATH`.
  Its state is one JSON file; it records calls and injects failures.
- Stand-ins and real code share the same constants.

## The gate

One `gate.sh` is the definition of "may land", the same locally and in CI:
`cargo fmt --check`, `./verify.sh`, `clippy -D warnings`, `cargo nextest run`.
It pins its toolchain and trusts nobody's `PATH`.

- Flux runs alone, before the tests: beside them it shares the cores, and its
  time measures them.
- It runs with `FLUXFLAGS="-Ftimings -Ffixpoint-timeout=20 -Fcatch-bugs"`: a
  solver query past twenty seconds fails the gate naming its function, every
  slow one is named in one run, and the slowest are reported. No limit on the
  whole run: one cut short names nothing.

## Release

- Pushing tag `v<version>` releases. The tag must equal `Cargo.toml`'s version.
- Every step is a subcommand of `packaging/release.sh`. CI only calls it.
- Build on stable for macOS (arm64, x86_64), Linux (arm64, x86_64) and Windows
  (x86_64). Before packaging, check each binary's arch and that `--version`
  says the release.
- Install is one line per platform:
  - macOS, Linux: `brew install <owner>/tap/<name>`
  - Windows: `scoop bucket add <owner> https://github.com/<owner>/scoop-bucket`,
    then `scoop install <name>`
- The Homebrew formula and the Scoop manifest are both generated from the built
  archives (url, hash, version), never written by hand, attached to the release
  and pushed to the tap and the bucket.
- Each says the same three things in its own words: the CLIs the tool runs
  (`depends_on` / `depends`), what the installer cannot install
  (`caveats` / `notes`), and a check that `--version` is the release
  (`test do`; for Scoop, in `release.sh` before packaging).

## Words and decisions

Terms go in `GLOSSARY.md` and decisions in `docs/adr/`: see `domain-modeling`.
Module and interface comments: see `comments-first`.

## Before you ship

- [ ] No pure crate does I/O; Flux says `0 trusted; 0 ignored`
- [ ] One JSON object per command; failure is `{"error"}` and non-zero
- [ ] An agent that never saw the tool runs its first command from `--help` alone
- [ ] `agent_help::check` passes in a test, and help names nothing the reader cannot open
- [ ] Every error carries the line that fixes it
- [ ] Killing it mid-run and running it again loses nothing
- [ ] The installed `--version` equals the tag
