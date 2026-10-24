# The manual repeats each command's usage and arguments

A command's answer to `--help` gives its text, then its usage and its arguments, read from clap's tree (#15). The manual, what the tool's own `--help` answers, says the same of every command: each command's place in it is that command's answer one heading level down, its usage and arguments included, so an agent that reads the manual once knows every flag of every command without asking again. A global argument is listed under every command it reaches, in the manual too.

## Considered Options

- **Commands' text only in the manual, their arguments only in their own answer**: a shorter manual, but an agent that read it must still run `<command> --help` once per command to learn its flags, and the manual is no longer everything the tool's help says.
- **Global arguments once, under the tool, not under each command**: shorter again, but a command's place in the manual would no longer be its answer, and an agent reading one command's section would miss the flags that reach it.

## Consequences

The manual grows with every argument, a global one once per command. Both forms come from one function, so a command's answer and its place in the manual cannot drift apart; a test holds that one is the other one level down.
