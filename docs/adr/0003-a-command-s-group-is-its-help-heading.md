# A command's group is its clap help heading, and clap answers `--help` with the manual

The manual lists commands grouped by who runs them, and the group must be declared where the subcommand is defined. Clap has one free per-subcommand heading, `next_help_heading`, which its derive sets with `#[command(next_help_heading = "...")]` on the variant or struct; `Help` reads it as the group. A nested subcommand is in its parent's group unless it declares its own, so `plan create` need not repeat what `plan` says. A visible subcommand with no group, its own or a parent's, is an error that names it.

`--help` is answered by clap itself: `Help::clap()` returns the tool's command with `override_help` set on every command, the tool's to the whole manual (to the index since ADR 0007) and each subcommand's to its own text, so `--help` works at any depth, even when required arguments are missing.

## Considered Options

- **The group as a line inside each command's text**: a convention nothing checks, and text to strip before the command's own.
- **`subcommand_help_heading` on the parent**: one heading for all of a parent's subcommands, not one per subcommand.
- **clap's `Command::ext`**: behind an unstable feature.
- **A `--help` flag of the tool's own, read after parsing**: a command whose required argument is missing fails to parse before the flag is read.

## Consequences

`next_help_heading` also becomes the heading of arguments declared after it in clap's own help, which the manual replaces. `-h` answers what `--help` does.
