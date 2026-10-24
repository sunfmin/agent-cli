# A tool names its binary once, at its crate root

A topic's reference renders `<binary> help <name>`, so every topic needs the binary's name. The crate names it once with `agent_help::binary!("...")`, which defines a hidden constant at its root that `#[derive(Topic)]` reads; a crate that derives a topic without it fails to compile.

## Considered Options

- **The binary in each `#[topic(...)]`**: one name copied onto every topic, free to drift.
- **`env!("CARGO_PKG_NAME")` or `CARGO_BIN_NAME`**: the package is not always named as its binary, and `CARGO_BIN_NAME` does not exist in a library crate, where a tool's topics may live.
- **The binary given at run time, by the assembled manual**: a reference could then not be a constant, and a message could not carry one without the manual at hand.

## Consequences

Topics derived in one crate share one binary. The manual (`Help::new`) also learns a name, from clap's command; that the two agree is for it to check.
