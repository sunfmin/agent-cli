# agent-help

The help of a command-line tool whose reader is an agent: written where each fact lives, assembled into one Markdown manual.

## Language

**Manual**:
Everything a tool's help says, in Markdown as written: its commands, its topics, its tables.
_Avoid_: docs, man page

**Topic**:
A piece of the manual that belongs to no single command, shown by `<binary> help <name>`.
_Avoid_: guide, page, article

**Topic reference**:
The command that shows a topic, which a tool's own messages carry instead of writing the topic's name out.
_Avoid_: link, pointer
