# A command's or topic's text is written as a page of its own, and its headings nest where the manual places it

One text is shown in two places: alone, under `# \`tool plan create\``, and in the whole manual, under `#### \`tool plan create\`` (a topic: `# \`tool help t\`` and `### \`tool help t\``). Its author writes it once, as a page of its own: sections from `##`, the `#` title left to the manual. `Help` keeps it as written when it is asked for alone, and in the whole manual moves every ATX heading outside a fenced code block down by the levels between a page's title and the heading it sits under, so the manual's outline is one tree and an agent never reads one command's `## When` as a section of the tool. A table's help, expanded into a text, moves with it. A command's usage and arguments (ADR 0005) are `##` sections of the same page and move with it, so they and the author's sections are one level.

A `#` heading in a text is an error naming where it is, as is one the manual would move past `######`: both answers fail, and `check` names them before release.

## Considered Options

- **Write the sections at the manual's depth, `#####`**: the text alone then opens below a missing level, and the author must know where the manual places it.
- **Clamp a heading past `######` to `######`**: two levels of the outline silently become one.
- **Leave a `#` heading as it is**: alone it is a second title; in the manual, a heading at the top of the tool.

## Consequences

A command's sections may go `###` deep and a topic's `####`; a deeper one is refused, not flattened. Fences are found by ```` ``` ```` and `~~~` alike, so `code_blocks` hands over blocks fenced with either.
