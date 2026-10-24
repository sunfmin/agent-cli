# `--help` answers the index, and `help all` the whole manual

An agent reads `<binary> --help` first, and until now that was the whole manual: afkconvoy's is 3458 lines, 340 KB. Claude Code keeps a command's output past about 30 KB in a file and shows the agent a 2 KB preview of it, so the one call that was to answer everything did not (#24). `--help` now answers the index: the tool's own text, usage and arguments, then each command's line under its group, in the order the whole manual has the groups, then each topic's line, then `<binary> help all`'s. A command's line is the first line of its `about` (else its `long_about`); a topic's is the first sentence of its Markdown, as a table entry's help opens its item. `<command> --help` and `help <topic>` answer as before; `help all` answers the whole manual, unchanged, the one a page for humans is generated from.

`Help::topic_help("all")` answers the whole manual, so every tool shares the name with no code of its own, and `check` fails on a topic named `all`, which that name would hide, and on a topic that does not open with a sentence, whose line would be empty. Both are `check`'s, not an answer's (ADR 0004): neither makes what the tool shows wrong.

## Considered Options

- **Keep the whole manual on `--help`** (agent-cli#5): one call for everything, but past the size an agent's harness shows, it is one call for a 2 KB preview.
- **The index without the tool's usage and arguments**: shorter, but the global arguments would then be shown only under each command, and the tool's own answer would differ from the head of the whole manual for no reader's gain.
- **A topic's line as a separate one-line summary attribute**: a second text to keep beside the first; the first sentence is already written, and `check` makes sure it is there.
- **`all` as a name each tool handles in its own `help` command**: every tool would repeat it, and could spell it differently.

## Consequences

An agent runs `<command> --help` before a command's first run, as the tool's own text tells it to; ADR 0005 still holds of the whole manual, now behind `help all`. The index grows by one line per command and topic, not by their text. A tool that depends on agent-help keeps answering as it did until it moves its pin.
