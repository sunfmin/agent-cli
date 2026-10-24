# agent-help

The help of a command-line tool whose reader is an AI agent: written where each fact lives, assembled into one Markdown manual.

## Language

**Manual**:
Everything a tool's help says, in Markdown as written: its commands, its topics, its tables.
_Avoid_: docs, man page

**Help**:
A fact's Markdown, as its author wrote it, for an agent to read.
_Avoid_: docs, description, explanation

**Topic**:
A piece of the manual that belongs to no single command, shown by `<binary> help <name>`.
_Avoid_: guide, page, article

**Topic reference**:
The command that shows a topic, which a tool's own messages carry instead of writing the topic's name out.
_Avoid_: link, pointer

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
