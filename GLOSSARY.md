# agent-help

The help of a command-line tool whose reader is an AI agent: written where each fact lives, assembled into one Markdown manual.

## Language

**Manual**:
Everything a tool's help says, in Markdown as written: its commands, its topics, its tables. `<binary> help all` answers it whole.
_Avoid_: docs, man page

**Index**:
What `<binary> --help` answers: the tool's own text, then each command's line (the first line of its text) under its group, then each topic's line (the first sentence of its Markdown).
_Avoid_: summary, table of contents

**Help**:
A fact's Markdown, as its author wrote it, for an agent to read.
_Avoid_: docs, description, explanation

**Topic**:
A piece of the manual that belongs to no single command, shown by `<binary> help <name>`.
_Avoid_: guide, page, article

**Topic reference**:
The command that shows a topic, which a tool's own messages carry instead of writing the topic's name out.
_Avoid_: link, pointer

**Topic link**:
`[...](help:<topic>)` written inside a help, sending the reader to a topic by its name.
_Avoid_: reference, pointer

**Table**:
A closed set of facts of one kind, such as a tool's refusal codes, whose every entry carries its help.
_Avoid_: registry, catalog

**Entry**:
One member of a table: its name, as a tool prints it, and its help.
_Avoid_: item, row

**Group**:
Who runs a command, such as "Run by the operator": the manual lists commands under it. A subcommand declares it where it is defined.
_Avoid_: category, section

**Placeholder**:
`{table:<name>}` or `{const:<NAME>}` written inside a help, standing for a table or a value the tool registered, so a fact is written once.
_Avoid_: variable, macro, template
