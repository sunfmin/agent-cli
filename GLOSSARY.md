# agent-help

The help of a command-line tool whose reader is an AI agent: written where
each fact lives, assembled into one Markdown manual.

## Language

**Help**:
A fact's Markdown, as its author wrote it, for an agent to read.
_Avoid_: docs, description, explanation

**Table**:
A closed set of facts of one kind, such as a tool's refusal codes, whose every entry carries its help.
_Avoid_: registry, catalog

**Entry**:
One member of a table: its name, as a tool prints it, and its help.
_Avoid_: item, row
